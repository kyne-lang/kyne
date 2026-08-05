//! The `Diagnostic` type and its rendering, per
//! docs/COMPILER_ARCHITECTURE.md §15.

use std::fmt::Write as _;

use crate::{Severity, Span};

/// A single `= help:` entry, optionally followed by a suggested-code
/// snippet, per docs/COMPILER_ARCHITECTURE.md §15's "Suggested Fix" field.
#[derive(Debug, Clone)]
struct Help {
    message: String,
    snippet: Vec<String>,
}

/// A compiler diagnostic carrying every field
/// docs/COMPILER_ARCHITECTURE.md §15 requires: Error Code, Title,
/// Explanation (the span + label), Reason (`notes`), Suggested Fix
/// (`helps`), and Future Documentation Link (`doc_link`).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    code: &'static str,
    severity: Severity,
    title: String,
    file: String,
    span: Span,
    label: String,
    notes: Vec<String>,
    helps: Vec<Help>,
    doc_link: Option<String>,
}

impl Diagnostic {
    /// Constructs a blocking Compiler Error. `code` MUST start with `KY`,
    /// per docs/COMPILER_ARCHITECTURE.md §15.1.
    pub fn error(
        code: &'static str,
        title: impl Into<String>,
        file: impl Into<String>,
        span: Span,
        label: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Error, code, title, file, span, label)
    }

    /// Constructs a non-blocking Security Analyzer warning. `code` MUST
    /// start with `KS`, per docs/COMPILER_ARCHITECTURE.md §15.1.
    pub fn warning(
        code: &'static str,
        title: impl Into<String>,
        file: impl Into<String>,
        span: Span,
        label: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Warning, code, title, file, span, label)
    }

    /// Constructs a non-blocking Security Analyzer hint. `code` MUST start
    /// with `KS`, per docs/COMPILER_ARCHITECTURE.md §15.1.
    pub fn hint(
        code: &'static str,
        title: impl Into<String>,
        file: impl Into<String>,
        span: Span,
        label: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Hint, code, title, file, span, label)
    }

    fn new(
        severity: Severity,
        code: &'static str,
        title: impl Into<String>,
        file: impl Into<String>,
        span: Span,
        label: impl Into<String>,
    ) -> Self {
        let namespace_ok = match severity {
            Severity::Error => code.starts_with("KY"),
            Severity::Warning | Severity::Hint => code.starts_with("KS"),
        };
        assert!(
            namespace_ok,
            "diagnostic code {code} does not match its severity {severity:?} - \
             per COMPILER_ARCHITECTURE.md §15.1, KY codes are blocking Compiler \
             Errors and KS codes are non-blocking Security Analyzer findings"
        );
        Diagnostic {
            code,
            severity,
            title: title.into(),
            file: file.into(),
            span,
            label: label.into(),
            notes: Vec::new(),
            helps: Vec::new(),
            doc_link: None,
        }
    }

    /// Adds a `= note:` line explaining why this is a problem in Kyne's
    /// domain specifically, per §15's "Reason" field.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Adds a `= help:` line with no attached code snippet.
    pub fn help(mut self, message: impl Into<String>) -> Self {
        self.helps.push(Help {
            message: message.into(),
            snippet: Vec::new(),
        });
        self
    }

    /// Adds a `= help:` line with an attached suggested-code snippet, per
    /// §15's "Suggested Fix" field.
    pub fn help_with_snippet<S: Into<String>>(
        mut self,
        message: impl Into<String>,
        snippet: impl IntoIterator<Item = S>,
    ) -> Self {
        self.helps.push(Help {
            message: message.into(),
            snippet: snippet.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Sets the Future Documentation Link field. Reserved for a future
    /// documentation-site integration, per §15.
    pub fn doc_link(mut self, link: impl Into<String>) -> Self {
        self.doc_link = Some(link.into());
        self
    }

    pub fn code(&self) -> &str {
        self.code
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn span(&self) -> Span {
        self.span
    }

    /// Renders this diagnostic in the terminal format shown throughout
    /// docs/COMPILER_ARCHITECTURE.md §15.2 and docs/LANGUAGE_SPEC.md §14.
    ///
    /// `source` MUST be the full source text of `self.file`, so the
    /// offending line can be quoted in context.
    pub fn render(&self, source: &str) -> String {
        let gutter = (self.span.line.to_string().len()).max(2);
        let line_text = source
            .lines()
            .nth((self.span.line - 1) as usize)
            .unwrap_or("");
        let mut out = String::new();

        writeln!(
            out,
            "{}[{}]: {}",
            self.severity.tag(),
            self.code,
            self.title
        )
        .unwrap();
        writeln!(
            out,
            "{:gutter$}--> {}:{}:{}",
            "",
            self.file,
            self.span.line,
            self.span.column,
            gutter = gutter
        )
        .unwrap();
        writeln!(out, "{:gutter$} |", "", gutter = gutter).unwrap();
        writeln!(
            out,
            "{:>gutter$} | {}",
            self.span.line,
            line_text,
            gutter = gutter
        )
        .unwrap();

        let indent = " ".repeat((self.span.column - 1) as usize);
        let carets = "^".repeat(self.span.length);
        writeln!(
            out,
            "{:gutter$} | {indent}{carets} {}",
            "",
            self.label,
            gutter = gutter
        )
        .unwrap();

        if !self.notes.is_empty() || !self.helps.is_empty() {
            writeln!(out, "{:gutter$} |", "", gutter = gutter).unwrap();
        }
        for note in &self.notes {
            writeln!(out, "{:gutter$} = note: {note}", "", gutter = gutter).unwrap();
        }
        for (i, help) in self.helps.iter().enumerate() {
            writeln!(
                out,
                "{:gutter$} = help: {}",
                "",
                help.message,
                gutter = gutter
            )
            .unwrap();
            if !help.snippet.is_empty() {
                writeln!(out, "{:gutter$} |", "", gutter = gutter).unwrap();
                for line in &help.snippet {
                    writeln!(out, "{:gutter$} |     {line}", "", gutter = gutter).unwrap();
                }
                if i + 1 < self.helps.len() {
                    writeln!(out, "{:gutter$} |", "", gutter = gutter).unwrap();
                }
            }
        }

        // Drop the final trailing newline `writeln!` leaves - callers that
        // print diagnostics one per line add their own separator.
        out.pop();
        out
    }
}
