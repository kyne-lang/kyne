## Summary

What does this change do, and why?

## Type of change

- [ ] Bug fix (no constitutional or ADR change required)
- [ ] Implementation of an Accepted KIP (link the KIP below)
- [ ] Implementation decision recorded in an ADR (link the ADR below)
- [ ] Documentation (informative document, per `GOVERNANCE.md` §17)
- [ ] Routine maintenance (per `GOVERNANCE.md` §26 — dependency updates, CI, etc.)

## Related KIP / ADR

Link the Accepted KIP or ADR this implements, if any. Per
[`GOVERNANCE.md` §12](../docs/GOVERNANCE.md#12-kips), a change to any
constitutional document's specified behavior requires a KIP to already be
Accepted before this pull request is opened, not the reverse.

## Checklist

Per [`ARCHITECTURE.md` §18](../ARCHITECTURE.md#18-contributor-workflow)
and [`GOVERNANCE.md` §13](../docs/GOVERNANCE.md#13-code-review-philosophy):

- [ ] Tests added or updated, per [`ARCHITECTURE.md` §16](../ARCHITECTURE.md#16-testing-philosophy)
- [ ] Documentation (doc comments, crate `README.md`, or constitutional document) updated to match this change
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] No new dependency was added without being named and justified in this description
- [ ] This change respects the dependency direction rules in [`ARCHITECTURE.md` §10](../ARCHITECTURE.md#10-dependency-rules) (no backward or cyclic dependency introduced)

## Additional context

Anything else a reviewer should know.
