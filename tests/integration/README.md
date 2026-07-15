# tests/integration/

End-to-end tests exercising the full pipeline from `.kyn` source through a
real Cargo/Soroban build, per
[`COMPILER_ARCHITECTURE.md` §19](../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy)'s
end-to-end category — the strongest available test category, confirming
generated code both compiles and executes with the exact behavior
`RUNTIME_MODEL.md` specifies.

No tests exist yet. Requires WASM generation to exist first — introduced
starting at the v0.3 release gate, per
[`ROADMAP.md` §25](../../docs/ROADMAP.md#25-release-gates).
