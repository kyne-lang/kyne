//! The pretty-printer: walks the CST directly (not the AST) and re-emits
//! canonical text, per docs/LANGUAGE_SPEC.md §13.
//!
//! Because `kyne fmt` recomputes formatting from the CST's structure
//! rather than preserving the author's original whitespace, this crate
//! does not need to reason about operator precedence when deciding
//! whether to print parentheses: any `ParenExpr` node the parser
//! produced is printed exactly where it is, and no new ones are ever
//! synthesized - §13 has no "remove redundant parens" rule (that is an
//! AST-lowering concern, per `kyne_ast`), so preserving them as written
//! is correct, not merely convenient.

use kyne_cst::{Cst, NodeKind, SyntaxNode, TokenKind};

const INDENT_WIDTH: usize = 4;
const MAX_WIDTH: usize = 100;

pub fn print(cst: &Cst) -> String {
    let mut printer = Printer {
        source: cst.source(),
        out: String::new(),
        indent: 0,
    };
    printer.print_program(cst.root());
    if !printer.out.ends_with('\n') {
        printer.out.push('\n');
    }
    printer.out
}

struct Printer<'a> {
    source: &'a str,
    out: String,
    indent: usize,
}

impl<'a> Printer<'a> {
    fn indent_str(&self) -> String {
        " ".repeat(self.indent * INDENT_WIDTH)
    }

    fn current_col(&self) -> usize {
        self.out.rsplit('\n').next().unwrap_or("").chars().count()
    }

    fn write(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn write_indent(&mut self) {
        let s = self.indent_str();
        self.write(&s);
    }

    fn newline(&mut self) {
        self.out.push('\n');
    }

    fn ensure_blank_line(&mut self) {
        // Collapse to exactly one blank line before what comes next,
        // regardless of how many blank lines (if any) trailed the
        // previous line already.
        while self.out.ends_with('\n') {
            self.out.pop();
        }
        self.write("\n\n");
    }

    // ---- Leading comments and blank-line detection ----

    /// Leading trivia attaches to the innermost node open at the time the
    /// next significant token is `bump`-ed, per `kyne_parser`'s tree-
    /// building design - so a statement's leading whitespace/comments
    /// typically live nested inside its first descendant (e.g. an
    /// `IdentExpr`), not as this node's own direct children. Walking
    /// `all_tokens()` (leftmost, document order, across the whole
    /// subtree) finds them regardless of nesting depth.
    fn leading_trivia<'b>(&self, node: &'b SyntaxNode) -> Vec<&'b kyne_lexer::Token> {
        node.all_tokens()
            .into_iter()
            .take_while(|t| {
                matches!(
                    t.kind,
                    TokenKind::Whitespace
                        | TokenKind::LineComment
                        | TokenKind::DocComment
                        | TokenKind::BlockComment
                )
            })
            .collect()
    }

    fn leading_comment_lines(&self, node: &SyntaxNode) -> Vec<String> {
        self.leading_trivia(node)
            .into_iter()
            .filter(|t| {
                matches!(
                    t.kind,
                    TokenKind::LineComment | TokenKind::DocComment | TokenKind::BlockComment
                )
            })
            .map(|t| t.text(self.source).to_string())
            .collect()
    }

    fn had_blank_line_before(&self, node: &SyntaxNode) -> bool {
        self.leading_trivia(node)
            .into_iter()
            .filter(|t| t.kind == TokenKind::Whitespace)
            .any(|t| t.text(self.source).matches('\n').count() >= 2)
    }

    fn write_leading_comments(&mut self, node: &SyntaxNode) {
        for line in self.leading_comment_lines(node) {
            self.write_indent();
            self.write(line.trim_end());
            self.newline();
        }
    }

    // ---- §2 Program ----

    fn print_program(&mut self, node: &SyntaxNode) {
        let mut imports: Vec<&SyntaxNode> = node.children_of_kind(NodeKind::Import).collect();
        imports.sort_by_key(|i| self.import_sort_key(i));
        for (idx, import) in imports.iter().enumerate() {
            if idx > 0 {
                self.newline();
            }
            self.print_import(import);
        }
        if !imports.is_empty() {
            self.newline();
        }

        let mut first = true;
        for item in node.nodes() {
            if item.kind == NodeKind::Import {
                continue;
            }
            if !first {
                self.newline();
            }
            first = false;
            self.print_top_level_decl(item);
        }
    }

    fn import_sort_key(&self, node: &SyntaxNode) -> String {
        node.child(NodeKind::ImportPath)
            .map(|p| identifiers(p, self.source).collect::<Vec<_>>().join("."))
            .unwrap_or_default()
    }

    fn print_import(&mut self, node: &SyntaxNode) {
        self.write_leading_comments(node);
        let path = node
            .child(NodeKind::ImportPath)
            .map(|p| identifiers(p, self.source).collect::<Vec<_>>().join("."))
            .unwrap_or_default();
        self.write(&format!("use {path}"));
        if let Some(list) = node.child(NodeKind::IdentifierList) {
            let names: Vec<String> = identifiers(list, self.source).collect();
            self.write(&format!(".{{{}}}", names.join(", ")));
        }
        self.write(";");
        self.newline();
    }

    fn print_top_level_decl(&mut self, node: &SyntaxNode) {
        self.write_leading_comments(node);
        match node.kind {
            NodeKind::ContractDecl => self.print_contract_decl(node),
            NodeKind::StructDecl => self.print_struct_decl(node),
            NodeKind::EnumDecl => self.print_enum_decl(node),
            NodeKind::ErrorDecl => self.print_error_decl(node),
            NodeKind::EventDecl => self.print_event_decl(node),
            NodeKind::ConstDecl => self.print_const_decl(node),
            NodeKind::FnDecl => self.print_fn_decl(node),
            _ => {}
        }
    }

    // ---- §3 Contracts ----

    fn print_contract_decl(&mut self, node: &SyntaxNode) {
        let name = first_identifier(node, self.source);
        self.write_indent();
        self.write(&format!("contract {name} {{"));
        self.newline();
        self.indent += 1;

        let members: Vec<&SyntaxNode> = node
            .nodes()
            .filter(|n| is_contract_member_kind(n.kind))
            .collect();
        let mut prev_category: Option<u8> = None;
        for member in members {
            let category = member_category(member, self.source);
            if let Some(prev) = prev_category {
                if prev != category || self.had_blank_line_before(member) {
                    self.ensure_blank_line();
                }
            }
            self.print_contract_member(member);
            prev_category = Some(category);
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}");
        self.newline();
    }

    fn print_contract_member(&mut self, node: &SyntaxNode) {
        self.write_leading_comments(node);
        match node.kind {
            NodeKind::StateDecl => self.print_state_decl(node),
            NodeKind::ConstDecl => self.print_const_decl(node),
            NodeKind::ErrorDecl => self.print_error_decl(node),
            NodeKind::EventDecl => self.print_event_decl(node),
            NodeKind::StructDecl => self.print_struct_decl(node),
            NodeKind::EnumDecl => self.print_enum_decl(node),
            NodeKind::FnDecl => self.print_fn_decl(node),
            _ => {}
        }
    }

    fn print_state_decl(&mut self, node: &SyntaxNode) {
        let name = first_identifier(node, self.source);
        let ty = self.render_type(
            node.child(NodeKind::Type)
                .expect("StateDecl always has a Type"),
        );
        self.write_indent();
        self.write(&format!("state {name}: {ty}"));
        if let Some(init) = expr_children(node).next() {
            let expr = self.render_expr(init);
            self.write(&format!(" = {expr}"));
        }
        self.write(";");
        self.newline();
    }

    fn print_const_decl(&mut self, node: &SyntaxNode) {
        let name = first_identifier(node, self.source);
        let ty = self.render_type(
            node.child(NodeKind::Type)
                .expect("ConstDecl always has a Type"),
        );
        let value = expr_children(node)
            .next()
            .map(|e| self.render_expr(e))
            .unwrap_or_default();
        self.write_indent();
        self.write(&format!("const {name}: {ty} = {value};"));
        self.newline();
    }

    fn print_error_decl(&mut self, node: &SyntaxNode) {
        let name = first_identifier(node, self.source);
        self.write_indent();
        match node.child(NodeKind::ErrorVariants) {
            None => {
                self.write(&format!("error {name};"));
                self.newline();
            }
            Some(variants_node) => {
                let variants: Vec<String> = identifiers(variants_node, self.source).collect();
                self.write(&format!("error {name} {{"));
                self.newline();
                self.indent += 1;
                for v in &variants {
                    self.write_indent();
                    self.write(v);
                    self.write(",");
                    self.newline();
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
                self.newline();
            }
        }
    }

    fn print_event_decl(&mut self, node: &SyntaxNode) {
        let name = first_identifier(node, self.source);
        let params = self.render_param_list(node.child(NodeKind::ParamList));
        self.write_indent();
        self.print_wrappable_list_header(&format!("event {name}"), "(", &params, ")");
        self.write(";");
        self.newline();
    }

    fn print_struct_decl(&mut self, node: &SyntaxNode) {
        let name = first_identifier(node, self.source);
        let fields = self.render_field_list(node.child(NodeKind::FieldList));
        self.write_indent();
        self.print_always_multiline_brace_block(&format!("struct {name}"), &fields);
    }

    fn print_enum_decl(&mut self, node: &SyntaxNode) {
        let name = first_identifier(node, self.source);
        let variants: Vec<&SyntaxNode> = node.children_of_kind(NodeKind::EnumVariant).collect();
        let rendered: Vec<String> = variants
            .iter()
            .map(|v| self.render_enum_variant(v))
            .collect();
        self.write_indent();
        self.print_always_multiline_brace_block(&format!("enum {name}"), &rendered);
    }

    /// Prints `header { items }`, always one item per line with a
    /// trailing comma - never collapsed to one line regardless of width.
    /// Used for `struct`/`enum`/`error` declaration bodies specifically:
    /// every one of the six canonical examples renders these
    /// declarations multi-line even when they would easily fit on one
    /// line, unlike a call's argument list or a struct-literal
    /// *expression*, both of which do collapse when they fit. This
    /// mirrors LANGUAGE_SPEC.md's own Auditability principle - a
    /// contract's storage/error surface reads as one declaration per
    /// line, always, so an auditor can `grep` it without first checking
    /// whether a given declaration happened to be short.
    fn print_always_multiline_brace_block(&mut self, header: &str, items: &[String]) {
        if items.is_empty() {
            self.write(header);
            self.write(" {}");
            self.newline();
            return;
        }
        self.write(header);
        self.write(" {");
        self.newline();
        self.indent += 1;
        for item in items {
            self.write_indent();
            self.write(item);
            self.write(",");
            self.newline();
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}");
        self.newline();
    }

    fn render_enum_variant(&mut self, node: &SyntaxNode) -> String {
        let name = first_identifier(node, self.source);
        if let Some(types) = node.child(NodeKind::TypeList) {
            let rendered: Vec<String> = types
                .children_of_kind(NodeKind::Type)
                .map(|t| self.render_type(t))
                .collect();
            format!("{name}({})", rendered.join(", "))
        } else if let Some(fields) = node.child(NodeKind::FieldList) {
            let rendered = self.render_field_list(Some(fields));
            format!("{name} {{ {} }}", rendered.join(", "))
        } else {
            name
        }
    }

    // ---- §5 Functions ----

    fn print_fn_decl(&mut self, node: &SyntaxNode) {
        let visibility = if node.tokens().any(|t| t.kind == TokenKind::Public) {
            "public fn "
        } else if node.tokens().any(|t| t.kind == TokenKind::Internal) {
            "internal fn "
        } else {
            "fn "
        };
        let name = first_identifier(node, self.source);
        let params = self.render_param_list(node.child(NodeKind::ParamList));
        let ret = node
            .child(NodeKind::Type)
            .map(|t| format!(" -> {}", self.render_type(t)))
            .unwrap_or_default();

        self.write_indent();
        let header = format!("{visibility}{name}");
        let suffix = format!("){ret} {{");
        self.print_wrappable_list_header_with_suffix(&header, "(", &params, &suffix);
        self.newline();
        self.indent += 1;
        if let Some(block) = node.child(NodeKind::Block) {
            self.print_block_statements(block);
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}");
        self.newline();
    }

    // ---- Types, fields, params (shared rendering) ----

    fn render_type(&mut self, node: &SyntaxNode) -> String {
        let name = node
            .tokens()
            .find(|t| t.kind == TokenKind::Identifier)
            .map(|t| t.text(self.source).to_string())
            .unwrap_or_default();
        let mut args = node.children_of_kind(NodeKind::Type);
        match name.as_str() {
            "list" => format!(
                "list<{}>",
                args.next().map(|t| self.render_type(t)).unwrap_or_default()
            ),
            "Option" => format!(
                "Option<{}>",
                args.next().map(|t| self.render_type(t)).unwrap_or_default()
            ),
            "map" => {
                let k = args.next().map(|t| self.render_type(t)).unwrap_or_default();
                let v = args.next().map(|t| self.render_type(t)).unwrap_or_default();
                format!("map<{k}, {v}>")
            }
            "Result" => {
                let k = args.next().map(|t| self.render_type(t)).unwrap_or_default();
                let v = args.next().map(|t| self.render_type(t)).unwrap_or_default();
                format!("Result<{k}, {v}>")
            }
            "bytes" if node.tokens().any(|t| t.kind == TokenKind::IntLiteral) => {
                let n = node
                    .tokens()
                    .find(|t| t.kind == TokenKind::IntLiteral)
                    .map(|t| t.text(self.source))
                    .unwrap_or_default();
                format!("bytes<{n}>")
            }
            _ => name,
        }
    }

    fn render_field_list(&mut self, node: Option<&SyntaxNode>) -> Vec<String> {
        match node {
            None => Vec::new(),
            Some(list) => list
                .children_of_kind(NodeKind::Field)
                .map(|f| {
                    let name = first_identifier(f, self.source);
                    let ty = f
                        .child(NodeKind::Type)
                        .map(|t| self.render_type(t))
                        .unwrap_or_default();
                    format!("{name}: {ty}")
                })
                .collect(),
        }
    }

    fn render_param_list(&mut self, node: Option<&SyntaxNode>) -> Vec<String> {
        match node {
            None => Vec::new(),
            Some(list) => list
                .children_of_kind(NodeKind::Param)
                .map(|p| {
                    let name = first_identifier(p, self.source);
                    let ty = p
                        .child(NodeKind::Type)
                        .map(|t| self.render_type(t))
                        .unwrap_or_default();
                    format!("{name}: {ty}")
                })
                .collect(),
        }
    }

    fn fits(&self, single_line_addition: &str) -> bool {
        self.current_col() + single_line_addition.chars().count() <= MAX_WIDTH
    }

    /// Prints `header(items)close_and_suffix`, wrapping to one item per
    /// line with a trailing comma when the flat form would exceed
    /// `MAX_WIDTH`, per LANGUAGE_SPEC.md §13's line-length and
    /// trailing-comma rules.
    fn print_wrappable_list_header_with_suffix(
        &mut self,
        header: &str,
        open: &str,
        items: &[String],
        suffix: &str,
    ) {
        let flat = format!("{header}{open}{}{suffix}", items.join(", "));
        if items.is_empty() || self.fits(&flat) {
            self.write(&flat);
            return;
        }
        self.write(header);
        self.write(open);
        self.newline();
        self.indent += 1;
        for item in items {
            self.write_indent();
            self.write(item);
            self.write(",");
            self.newline();
        }
        self.indent -= 1;
        self.write_indent();
        self.write(suffix);
    }

    fn print_wrappable_list_header(
        &mut self,
        header: &str,
        open: &str,
        items: &[String],
        close: &str,
    ) {
        self.print_wrappable_list_header_with_suffix(header, open, items, close);
    }

    // ---- §8 Statements ----

    fn print_block_statements(&mut self, node: &SyntaxNode) {
        let statements: Vec<&SyntaxNode> = node.nodes().collect();
        for (idx, stmt) in statements.iter().enumerate() {
            if idx > 0 && self.had_blank_line_before(stmt) {
                self.ensure_blank_line();
            }
            self.write_leading_comments(stmt);
            self.print_stmt(stmt);
        }
    }

    fn print_stmt(&mut self, node: &SyntaxNode) {
        self.write_indent();
        match node.kind {
            NodeKind::LetStmt => {
                let name = first_identifier(node, self.source);
                let ty = node
                    .child(NodeKind::Type)
                    .map(|t| format!(": {}", self.render_type(t)))
                    .unwrap_or_default();
                self.write(&format!("let {name}{ty} = "));
                let value = expr_children(node)
                    .next()
                    .map(|e| self.render_expr_fit(e, 1))
                    .unwrap_or_default();
                self.write(&format!("{value};"));
            }
            NodeKind::AssignStmt => {
                let mut exprs = expr_children(node);
                let target = exprs
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let value = exprs
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let op = assign_op_text(node);
                self.write(&format!("{target} {op} {value};"));
            }
            NodeKind::AuthStmt => {
                let value = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                self.write(&format!("auth({value});"));
            }
            NodeKind::EmitStmt => {
                let event = first_identifier(node, self.source);
                let args = self.render_arg_list(node.child(NodeKind::ArgList));
                self.write(&format!("emit {event}({});", args.join(", ")));
            }
            NodeKind::ThrowStmt => {
                let value = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                self.write(&format!("throw {value};"));
            }
            NodeKind::IfStmt => self.print_if(node),
            NodeKind::MatchStmt => self.print_match(node),
            NodeKind::ForStmt => {
                let var = first_identifier(node, self.source);
                let iterable = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                self.write(&format!("for {var} in {iterable} {{"));
                self.newline();
                self.indent += 1;
                if let Some(block) = node.child(NodeKind::Block) {
                    self.print_block_statements(block);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
            }
            NodeKind::WhileStmt => {
                let condition = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                self.write(&format!("while {condition} {{"));
                self.newline();
                self.indent += 1;
                if let Some(block) = node.child(NodeKind::Block) {
                    self.print_block_statements(block);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
            }
            NodeKind::ReturnStmt => {
                let value = expr_children(node).next().map(|e| self.render_expr(e));
                match value {
                    Some(v) => self.write(&format!("return {v};")),
                    None => self.write("return;"),
                }
            }
            NodeKind::BreakStmt => self.write("break;"),
            NodeKind::ContinueStmt => self.write("continue;"),
            NodeKind::ExprStmt => {
                let has_semicolon = node.tokens().any(|t| t.kind == TokenKind::Semicolon);
                let value = expr_children(node)
                    .next()
                    .map(|e| self.render_expr_fit(e, if has_semicolon { 1 } else { 0 }))
                    .unwrap_or_default();
                self.write(&value);
                if has_semicolon {
                    self.write(";");
                }
            }
            _ => {}
        }
        self.newline();
    }

    fn print_if(&mut self, node: &SyntaxNode) {
        let condition = expr_children(node)
            .next()
            .map(|e| self.render_expr(e))
            .unwrap_or_default();
        self.write(&format!("if {condition} {{"));
        self.newline();
        self.indent += 1;
        let blocks: Vec<&SyntaxNode> = node.children_of_kind(NodeKind::Block).collect();
        if let Some(then_block) = blocks.first() {
            self.print_block_statements(then_block);
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}");
        if let Some(nested) = node.child(NodeKind::IfStmt) {
            self.write(" else ");
            self.print_if(nested);
        } else if let Some(else_block) = blocks.get(1) {
            self.write(" else {");
            self.newline();
            self.indent += 1;
            self.print_block_statements(else_block);
            self.indent -= 1;
            self.write_indent();
            self.write("}");
        }
    }

    fn print_match(&mut self, node: &SyntaxNode) {
        let subject = expr_children(node)
            .next()
            .map(|e| self.render_expr(e))
            .unwrap_or_default();
        self.write(&format!("match {subject} {{"));
        self.newline();
        self.indent += 1;
        for arm in node.children_of_kind(NodeKind::MatchArm) {
            self.print_match_arm(arm);
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}");
    }

    fn print_match_arm(&mut self, node: &SyntaxNode) {
        self.write_indent();
        let pattern = node
            .child(NodeKind::Pattern)
            .map(|p| self.render_pattern(p))
            .unwrap_or_default();
        self.write(&pattern);
        let has_guard = node.tokens().any(|t| t.kind == TokenKind::If);
        let mut exprs = expr_children(node);
        if has_guard {
            let guard = exprs
                .next()
                .map(|e| self.render_expr(e))
                .unwrap_or_default();
            self.write(&format!(" if {guard}"));
        }
        self.write(" => ");
        if let Some(block) = node.child(NodeKind::Block) {
            self.write("{");
            self.newline();
            self.indent += 1;
            self.print_block_statements(block);
            self.indent -= 1;
            self.write_indent();
            self.write("}");
        } else if let Some(ret) = node.child(NodeKind::ReturnStmt) {
            match expr_children(ret).next().map(|e| self.render_expr(e)) {
                Some(v) => self.write(&format!("return {v}")),
                None => self.write("return"),
            }
            self.write(",");
        } else if let Some(thr) = node.child(NodeKind::ThrowStmt) {
            let value = expr_children(thr)
                .next()
                .map(|e| self.render_expr(e))
                .unwrap_or_default();
            self.write(&format!("throw {value},"));
        } else if node.child(NodeKind::BreakStmt).is_some() {
            self.write("break,");
        } else if node.child(NodeKind::ContinueStmt).is_some() {
            self.write("continue,");
        } else {
            let value = exprs
                .next()
                .map(|e| self.render_expr(e))
                .unwrap_or_default();
            self.write(&format!("{value},"));
        }
        self.newline();
    }

    fn render_pattern(&mut self, node: &SyntaxNode) -> String {
        if let Some(lit) = node.tokens().find(|t| {
            matches!(
                t.kind,
                TokenKind::IntLiteral
                    | TokenKind::StringLiteral
                    | TokenKind::True
                    | TokenKind::False
            )
        }) {
            let sign = if node.tokens().any(|t| t.kind == TokenKind::Minus) {
                "-"
            } else {
                ""
            };
            return format!("{sign}{}", lit.text(self.source));
        }
        let names: Vec<String> = identifiers(node, self.source).collect();
        if names.first().map(String::as_str) == Some("_") {
            return "_".to_string();
        }
        let path = names.join("::");
        if let Some(list) = node.child(NodeKind::PatternList) {
            let rendered: Vec<String> = list
                .children_of_kind(NodeKind::Pattern)
                .map(|p| self.render_pattern(p))
                .collect();
            format!("{path}({})", rendered.join(", "))
        } else if let Some(fields) = node.child(NodeKind::FieldPatternList) {
            let rendered: Vec<String> = identifiers(fields, self.source).collect();
            format!("{path} {{ {} }}", rendered.join(", "))
        } else {
            path
        }
    }

    // ---- §7 Expressions ----

    /// Renders `node` flat if it fits within `MAX_WIDTH` from the
    /// current output column (accounting for `trailing_cols` more
    /// characters that will follow it on the same line, e.g. a `;`),
    /// otherwise falls back to a width-driven multi-line form for the
    /// node kinds that have one (`StructLiteral`, `IfStmt`-as-expression,
    /// a call/method-call whose last argument is a `StructLiteral`).
    /// Every other kind has no wrapped form and is returned flat
    /// regardless of width - LANGUAGE_SPEC.md §13 doesn't specify a
    /// general expression-wrapping algorithm, only the specific
    /// multi-line forms these three cases need per the canonical
    /// examples' own conformance requirement.
    fn render_expr_fit(&mut self, node: &SyntaxNode, trailing_cols: usize) -> String {
        let flat = self.render_expr(node);
        if self.current_col() + flat.chars().count() + trailing_cols <= MAX_WIDTH {
            return flat;
        }
        match node.kind {
            NodeKind::StructLiteral => self.render_struct_literal_wrapped(node),
            NodeKind::IfStmt => self.render_if_expr_wrapped(node),
            NodeKind::CallExpr | NodeKind::MethodCallExpr => {
                self.render_call_wrapped(node).unwrap_or(flat)
            }
            _ => flat,
        }
    }

    fn render_struct_literal_wrapped(&mut self, node: &SyntaxNode) -> String {
        let path_node = expr_children(node).next();
        let path = match path_node {
            Some(p) if p.kind == NodeKind::PathExpr => {
                identifiers(p, self.source).collect::<Vec<_>>().join("::")
            }
            Some(p) => first_identifier(p, self.source),
            None => String::new(),
        };
        let fields: Vec<&SyntaxNode> = node.children_of_kind(NodeKind::FieldInit).collect();
        let mut out = format!("{path} {{\n");
        self.indent += 1;
        for f in &fields {
            out.push_str(&self.indent_str());
            out.push_str(&self.render_field_init(f));
            out.push_str(",\n");
        }
        self.indent -= 1;
        out.push_str(&self.indent_str());
        out.push('}');
        out
    }

    /// A call/method-call whose *last* argument is a `StructLiteral`
    /// wraps by "hugging" that argument: everything up to and including
    /// the struct type name stays on the call's own line, the fields
    /// indent one level deeper, and the closing `}` shares a line with
    /// the call's closing `)` - the same pattern `rustfmt` uses for a
    /// trailing closure or struct-literal argument. Returns `None` when
    /// the call has no such trailing struct-literal argument to hug,
    /// since there is no other general call-wrapping form implemented.
    fn render_call_wrapped(&mut self, node: &SyntaxNode) -> Option<String> {
        let (callee_text, args): (String, Vec<&SyntaxNode>) = match node.kind {
            NodeKind::CallExpr => {
                let callee = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let args = node
                    .child(NodeKind::ArgList)
                    .map(|l| expr_children(l).collect())
                    .unwrap_or_default();
                (callee, args)
            }
            NodeKind::MethodCallExpr => {
                let base = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let method = identifiers(node, self.source).last().unwrap_or_default();
                let args = node
                    .child(NodeKind::ArgList)
                    .map(|l| expr_children(l).collect())
                    .unwrap_or_default();
                (format!("{base}.{method}"), args)
            }
            _ => return None,
        };
        let (last, rest) = args.split_last()?;
        if last.kind != NodeKind::StructLiteral {
            return None;
        }
        let rest_flat: Vec<String> = rest.iter().map(|a| self.render_expr(a)).collect();
        let prefix = if rest_flat.is_empty() {
            format!("{callee_text}(")
        } else {
            format!("{callee_text}({}, ", rest_flat.join(", "))
        };
        let wrapped_struct = self.render_struct_literal_wrapped(last);
        Some(format!("{prefix}{wrapped_struct})"))
    }

    fn render_if_expr_wrapped(&mut self, node: &SyntaxNode) -> String {
        let condition = expr_children(node)
            .next()
            .map(|e| self.render_expr(e))
            .unwrap_or_default();
        let blocks: Vec<&SyntaxNode> = node.children_of_kind(NodeKind::Block).collect();
        let mut out = format!("if {condition} {{\n");
        self.indent += 1;
        self.append_wrapped_branch_value(&mut out, blocks.first().copied());
        self.indent -= 1;
        out.push_str(&self.indent_str());
        out.push_str("} else {\n");
        self.indent += 1;
        if let Some(nested) = node.child(NodeKind::IfStmt) {
            out.push_str(&self.indent_str());
            out.push_str(&self.render_if_expr_wrapped(nested));
            out.push('\n');
        } else {
            self.append_wrapped_branch_value(&mut out, blocks.get(1).copied());
        }
        self.indent -= 1;
        out.push_str(&self.indent_str());
        out.push('}');
        out
    }

    fn append_wrapped_branch_value(&mut self, out: &mut String, block: Option<&SyntaxNode>) {
        let Some(value_node) = block
            .and_then(|b| b.nodes().next())
            .and_then(|stmt| expr_children(stmt).next())
        else {
            return;
        };
        out.push_str(&self.indent_str());
        let rendered = self.render_expr_fit(value_node, 0);
        out.push_str(&rendered);
        out.push('\n');
    }

    fn render_expr(&mut self, node: &SyntaxNode) -> String {
        match node.kind {
            NodeKind::ParenExpr => {
                let inner = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                format!("({inner})")
            }
            NodeKind::LiteralExpr => node
                .tokens()
                .find(|t| {
                    matches!(
                        t.kind,
                        TokenKind::IntLiteral
                            | TokenKind::StringLiteral
                            | TokenKind::True
                            | TokenKind::False
                    )
                })
                .map(|t| t.text(self.source).to_string())
                .unwrap_or_default(),
            NodeKind::IdentExpr => first_identifier(node, self.source),
            NodeKind::PathExpr => identifiers(node, self.source)
                .collect::<Vec<_>>()
                .join("::"),
            NodeKind::BinaryExpr => {
                let mut exprs = expr_children(node);
                let left = exprs
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let right = exprs
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let op = binary_op_text(node);
                format!("{left} {op} {right}")
            }
            NodeKind::UnaryExpr => {
                let operand = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let op = if node.tokens().any(|t| t.kind == TokenKind::Bang) {
                    "!"
                } else {
                    "-"
                };
                format!("{op}{operand}")
            }
            NodeKind::CallExpr => {
                let callee = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let args = self.render_arg_list(node.child(NodeKind::ArgList));
                format!("{callee}({})", args.join(", "))
            }
            NodeKind::FieldExpr => {
                let base = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let name = identifiers(node, self.source).last().unwrap_or_default();
                format!("{base}.{name}")
            }
            NodeKind::MethodCallExpr => {
                let base = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                let method = identifiers(node, self.source).last().unwrap_or_default();
                let args = self.render_arg_list(node.child(NodeKind::ArgList));
                format!("{base}.{method}({})", args.join(", "))
            }
            NodeKind::TryExpr => {
                let operand = expr_children(node)
                    .next()
                    .map(|e| self.render_expr(e))
                    .unwrap_or_default();
                format!("{operand}?")
            }
            NodeKind::StructLiteral => {
                let path_node = expr_children(node).next();
                let path = match path_node {
                    Some(p) => match p.kind {
                        NodeKind::PathExpr => {
                            identifiers(p, self.source).collect::<Vec<_>>().join("::")
                        }
                        _ => first_identifier(p, self.source),
                    },
                    None => String::new(),
                };
                let fields: Vec<String> = node
                    .children_of_kind(NodeKind::FieldInit)
                    .map(|f| self.render_field_init(f))
                    .collect();
                if fields.is_empty() {
                    format!("{path} {{}}")
                } else {
                    format!("{path} {{ {} }}", fields.join(", "))
                }
            }
            NodeKind::ListLiteral => {
                let items: Vec<String> = expr_children(node).map(|e| self.render_expr(e)).collect();
                format!("[{}]", items.join(", "))
            }
            NodeKind::MapLiteral => {
                let entries: Vec<String> = node
                    .children_of_kind(NodeKind::MapEntry)
                    .map(|e| self.render_map_entry(e))
                    .collect();
                format!(
                    "{{{}}}",
                    if entries.is_empty() {
                        String::new()
                    } else {
                        format!(" {} ", entries.join(", "))
                    }
                )
            }
            NodeKind::IfStmt => self.render_if_expr(node),
            NodeKind::MatchStmt => self.render_match_expr(node),
            _ => String::new(),
        }
    }

    /// `if`/`match` in expression position render on one line when
    /// possible (they only appear here inline within a larger
    /// expression, e.g. a `let` initializer) - full statement-shaped
    /// `if`/`match` go through `print_if`/`print_match` instead.
    fn render_if_expr(&mut self, node: &SyntaxNode) -> String {
        let condition = expr_children(node)
            .next()
            .map(|e| self.render_expr(e))
            .unwrap_or_default();
        let blocks: Vec<&SyntaxNode> = node.children_of_kind(NodeKind::Block).collect();
        let then_val = blocks
            .first()
            .map(|b| self.render_single_expr_block(b))
            .unwrap_or_default();
        let else_val = if let Some(nested) = node.child(NodeKind::IfStmt) {
            self.render_if_expr(nested)
        } else {
            blocks
                .get(1)
                .map(|b| self.render_single_expr_block(b))
                .unwrap_or_default()
        };
        format!("if {condition} {{ {then_val} }} else {{ {else_val} }}")
    }

    fn render_match_expr(&mut self, node: &SyntaxNode) -> String {
        // Expression-position `match` still needs a multi-line rendering
        // in the common case (multiple arms); fall back to the
        // statement-shaped printer's layout, captured into a string.
        let saved_indent = self.indent;
        let saved_len = self.out.len();
        self.print_match(node);
        let rendered = self.out[saved_len..].to_string();
        self.out.truncate(saved_len);
        self.indent = saved_indent;
        rendered
    }

    fn render_single_expr_block(&mut self, node: &SyntaxNode) -> String {
        match node.nodes().next() {
            Some(stmt) if stmt.kind == NodeKind::ExprStmt => expr_children(stmt)
                .next()
                .map(|e| self.render_expr(e))
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn render_field_init(&mut self, node: &SyntaxNode) -> String {
        let name = first_identifier(node, self.source);
        match expr_children(node).next() {
            Some(e) => format!("{name}: {}", self.render_expr(e)),
            None => name,
        }
    }

    fn render_map_entry(&mut self, node: &SyntaxNode) -> String {
        let mut exprs = expr_children(node);
        let key = exprs
            .next()
            .map(|e| self.render_expr(e))
            .unwrap_or_default();
        let value = exprs
            .next()
            .map(|e| self.render_expr(e))
            .unwrap_or_default();
        format!("{key}: {value}")
    }

    fn render_arg_list(&mut self, node: Option<&SyntaxNode>) -> Vec<String> {
        match node {
            None => Vec::new(),
            Some(list) => expr_children(list).map(|e| self.render_expr(e)).collect(),
        }
    }
}

fn is_contract_member_kind(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::StateDecl
            | NodeKind::ConstDecl
            | NodeKind::ErrorDecl
            | NodeKind::EventDecl
            | NodeKind::StructDecl
            | NodeKind::EnumDecl
            | NodeKind::FnDecl
    )
}

/// The canonical member-order category, per LANGUAGE_SPEC.md §3.2's nine
/// categories - used only to decide where a mandatory blank line goes,
/// per §13's blank-line rule. Does not itself validate or enforce
/// ordering: that is `kyne_semantics`'s job (a later stage this crate
/// deliberately does not depend on), so a member already out of order
/// is formatted in place, not reordered.
fn member_category(node: &SyntaxNode, source: &str) -> u8 {
    match node.kind {
        NodeKind::StateDecl => 1,
        NodeKind::ConstDecl => 2,
        NodeKind::ErrorDecl => 3,
        NodeKind::EventDecl => 4,
        NodeKind::StructDecl | NodeKind::EnumDecl => 5,
        NodeKind::FnDecl => {
            if first_identifier(node, source) == "init" {
                6
            } else if node.tokens().any(|t| t.kind == TokenKind::Public) {
                7
            } else if node.tokens().any(|t| t.kind == TokenKind::Internal) {
                8
            } else {
                9
            }
        }
        _ => 0,
    }
}

fn is_expr_kind(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::BinaryExpr
            | NodeKind::UnaryExpr
            | NodeKind::CallExpr
            | NodeKind::FieldExpr
            | NodeKind::MethodCallExpr
            | NodeKind::TryExpr
            | NodeKind::PathExpr
            | NodeKind::IdentExpr
            | NodeKind::LiteralExpr
            | NodeKind::ParenExpr
            | NodeKind::StructLiteral
            | NodeKind::ListLiteral
            | NodeKind::MapLiteral
            | NodeKind::IfStmt
            | NodeKind::MatchStmt
    )
}

fn expr_children(node: &SyntaxNode) -> impl Iterator<Item = &SyntaxNode> {
    node.nodes().filter(|n| is_expr_kind(n.kind))
}

fn identifiers<'a>(node: &'a SyntaxNode, source: &'a str) -> impl Iterator<Item = String> + 'a {
    node.tokens()
        .filter(|t| t.kind == TokenKind::Identifier)
        .map(|t| t.text(source).to_string())
}

fn first_identifier(node: &SyntaxNode, source: &str) -> String {
    identifiers(node, source).next().unwrap_or_default()
}

fn assign_op_text(node: &SyntaxNode) -> &'static str {
    node.tokens()
        .find_map(|t| match t.kind {
            TokenKind::Eq => Some("="),
            TokenKind::PlusEq => Some("+="),
            TokenKind::MinusEq => Some("-="),
            TokenKind::StarEq => Some("*="),
            TokenKind::SlashEq => Some("/="),
            TokenKind::PercentEq => Some("%="),
            _ => None,
        })
        .unwrap_or("=")
}

fn binary_op_text(node: &SyntaxNode) -> &'static str {
    node.tokens()
        .find_map(|t| match t.kind {
            TokenKind::Plus => Some("+"),
            TokenKind::Minus => Some("-"),
            TokenKind::Star => Some("*"),
            TokenKind::Slash => Some("/"),
            TokenKind::Percent => Some("%"),
            TokenKind::EqEq => Some("=="),
            TokenKind::NotEq => Some("!="),
            TokenKind::LAngle => Some("<"),
            TokenKind::RAngle => Some(">"),
            TokenKind::LtEq => Some("<="),
            TokenKind::GtEq => Some(">="),
            TokenKind::AmpAmp => Some("&&"),
            TokenKind::PipePipe => Some("||"),
            _ => None,
        })
        .unwrap_or("+")
}
