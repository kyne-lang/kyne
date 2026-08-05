# Deploying a Kyne Contract: A Basic Workflow

This guide walks through the full path from a `.kyn` source file to a
running contract instance on a local Soroban network, per
[`RUNTIME_MODEL.md` §3](../RUNTIME_MODEL.md#3-project-lifecycle)'s
project lifecycle (`.kyn` source → Compiler → Generated Rust → Soroban
Toolchain → WASM → Deployment → Contract Instance). It uses `Counter`
([`examples/canonical/counter.kyn`](../../examples/canonical/counter.kyn))
as its worked example — the simplest of the six canonical contracts.

**What's verified versus what isn't.** Every step through "Cargo Build"
(producing a real `.wasm` file) was run for real during issue #18's own
work and is marked ✅ below. Deployment itself needs the `stellar` CLI,
which was not available in the environment this guide was written in
(see [ADR-0013](../adr/ADR-0013-driver-build-integration.md) for
exactly why) — those steps are marked ⚠️ **unverified** and are written
from the Stellar CLI's own published command reference, not from a
real run. If a step's output doesn't match what's shown here, trust
your own terminal and `stellar --help` over this document, and please
open an issue.

## Prerequisites

- A Rust toolchain with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`.
- The [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools#cli) (`stellar`), which provides the Soroban Build, local-network, and deployment commands used below. Install per its own instructions — this guide does not duplicate them, since they change independently of Kyne.

## 1. Compile `.kyn` to Rust, then to WASM ✅ verified

`kyne build`'s eventual CLI form is specified in
[`TOOLCHAIN.md` §"`kyne build`"](../TOOLCHAIN.md#kyne-build---release) —
until that subcommand exists (`kyne_cli`'s own dispatch is separate,
in-progress work), the same pipeline is reachable today as a library,
through `kyne_driver`:

```rust
let output = kyne_driver::build_project(
    &std::fs::read_to_string("counter.kyn")?,
    "counter.kyn",
    std::path::Path::new("build/rust"), // mirrors TOOLCHAIN.md §14's build/rust/ artifact path
    true, // --release
)?;

println!("Cargo Build artifact: {}", output.cargo_wasm.display());
match &output.soroban_wasm {
    Some(path) => println!("Soroban Build artifact: {}", path.display()),
    None => println!("Soroban Build skipped: `stellar`/`soroban` CLI not found on PATH"),
}
```

This does four things, matching
[`COMPILER_ARCHITECTURE.md` §3](../COMPILER_ARCHITECTURE.md#3-compiler-pipeline)'s
pipeline exactly:

1. Runs the full compiler pipeline (`kyne_driver::compile`) — parse through `kyne_codegen` — producing formatted, idiomatic Rust source text.
2. Writes a complete, buildable Cargo crate (`Cargo.toml` + `src/lib.rs`) to `build/rust/`.
3. **Cargo Build**: runs `cargo build --target wasm32-unknown-unknown --release` against it — an ordinary, unmodified `cargo` invocation, per [`RUNTIME_MODEL.md` §3](../RUNTIME_MODEL.md#3-project-lifecycle).
4. **Soroban Build**: runs `stellar contract optimize` on the result, if `stellar` (or the older `soroban`) CLI is on `PATH`.

Run directly (equivalent to the snippet above, using this repository's own test coverage):

```sh
cargo test -p kyne_driver -- --ignored counter_builds_to_a_real_wasm_artifact
```

This was run for real during issue #18's work: **both Counter and
Token compile to a real, valid WASM binary** against the real
`soroban-sdk` (verified by reading the file's `\0asm` magic number —
see [ADR-0012](../adr/ADR-0012-codegen-implementation.md) and
[ADR-0013](../adr/ADR-0013-driver-build-integration.md) for the full
verification record, including the one real bug this process found and
fixed).

## 2. Start a local Soroban network ⚠️ unverified

```sh
stellar network start local --limits testnet
stellar network use local
```

This runs a local Stellar node with the Soroban host enabled, letting
you deploy and invoke contracts without touching Testnet or Mainnet.

## 3. Fund a source account ⚠️ unverified

```sh
stellar keys generate deployer --network local
stellar keys fund deployer --network local
```

`deployer` is a local alias the Stellar CLI resolves to a real keypair
it manages — every command below that takes `--source deployer` is
signing with that keypair.

## 4. Deploy the WASM ⚠️ unverified

Deploy whichever WASM step 1 produced — the Soroban Build output
(`output.soroban_wasm`) if available, otherwise the plain Cargo Build
output (`output.cargo_wasm`); both are valid, deployable WASM, per
[`RUNTIME_MODEL.md` §3](../RUNTIME_MODEL.md#3-project-lifecycle) — the
Soroban Build stage only optimizes size and embeds metadata, it does
not change the contract's behavior:

```sh
stellar contract deploy \
  --wasm build/rust/target/wasm32-unknown-unknown/release/counter.wasm \
  --source deployer \
  --network local
```

This prints the deployed contract's ID (a `C...`-prefixed address) —
save it for the next step. Per
[`RUNTIME_MODEL.md` §3](../RUNTIME_MODEL.md#3-project-lifecycle), this
is the point Counter becomes "a real, addressable, on-chain entity."

Counter's `init` function ([`LANGUAGE_SPEC.md` §3.5](../LANGUAGE_SPEC.md#35-the-constructor))
runs automatically as part of deployment, per the Soroban constructor
mechanism the compiler generates for it (see
[ADR-0011](../adr/ADR-0011-rir-implementation.md)) — there is no
separate "call init" step.

## 5. Invoke the deployed contract ⚠️ unverified

```sh
stellar contract invoke \
  --id <the C... address from step 4> \
  --source deployer \
  --network local \
  -- \
  increment --by <a G...-prefixed address>
```

```sh
stellar contract invoke \
  --id <the C... address from step 4> \
  --source deployer \
  --network local \
  -- \
  get
```

Everything after the bare `--` is Counter's own ABI — the Stellar CLI
derives it directly from the contract metadata `#[contractimpl]`
embeds, per [`LANGUAGE_SPEC.md` §3.3](../LANGUAGE_SPEC.md#33-the-public-api)'s
"a `public fn` on a contract becomes part of that contract's on-chain
interface."

## Future work

- Once `kyne_cli`'s subcommand dispatch exists, step 1 becomes literally
  `kyne build --release`, per
  [`TOOLCHAIN.md` §"`kyne build`"](../TOOLCHAIN.md#kyne-build---release) —
  this guide's library-call form will still work underneath it
  unchanged, since the CLI is specified to be "composed entirely from
  `kyne_driver`" (see `compiler/driver/README.md`).
- Steps 2 through 5 should be re-verified for real, and this guide's
  ⚠️ markers removed, the first time this repository is built somewhere
  with the Stellar CLI installed — see
  [ADR-0013](../adr/ADR-0013-driver-build-integration.md)'s
  Verification section for exactly what's still open.
