//! `kyne_fuzz` - fuzz-testing harnesses for `kyne_lexer` and
//! `kyne_parser`, per [`ROADMAP.md` §15](../../../docs/ROADMAP.md#15-security-roadmap):
//! "both are classic, high-value fuzz targets, and fuzzing them early
//! catches crash-class defects ... before later stages are built on top
//! of a potentially fragile foundation."
//!
//! This crate is the shared engine (seedable PRNG, byte mutator, seed
//! corpus, panic-catching runner) both `tests/lexer_fuzz.rs` and
//! `tests/parser_fuzz.rs` drive against their own target. See
//! docs/adr/ADR-0014-fuzz-harness-implementation.md for why a small,
//! dependency-free, `cargo test`-integrated harness was chosen over
//! `cargo-fuzz`/libFuzzer.

use std::panic::{self, AssertUnwindSafe};

/// A small, seedable, non-cryptographic PRNG (xorshift64*) - not meant
/// to resist prediction, only to make every fuzz run reproducible from
/// its seed: a failing input can always be reproduced by rerunning
/// [`run_fuzz_loop`] with the same seed.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // xorshift64* requires a nonzero state.
        Rng(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() as usize) % bound
        }
    }

    pub fn next_u8(&mut self) -> u8 {
        self.next_u64() as u8
    }
}

/// The largest a mutated input is allowed to grow. Without a cap, the
/// duplicate operation below can roughly double `bytes`' length on
/// every hit - repeated application (as [`run_fuzz_loop`] does, many
/// mutations per iteration, many iterations per run) reaches gigabytes
/// within a few dozen doublings, which in practice presents as a hang
/// (huge allocations and O(n) `Vec::insert` shifting), not a clean
/// out-of-memory abort. This was caught by this crate's own
/// `mutate_never_panics_on_repeated_application` test hanging during
/// issue #19's own verification, before either fuzz harness had even
/// reached the real lexer/parser - see
/// docs/adr/ADR-0014-fuzz-harness-implementation.md.
const MAX_LEN: usize = 64 * 1024;

/// Applies one mutation to `bytes` at a randomly chosen position, drawn
/// from a small, classic byte-fuzzing repertoire: flip a byte, insert a
/// random byte, delete a byte, or duplicate a slice elsewhere. This is
/// deliberately simple - a random, non-coverage-guided mutator - since
/// this crate's goal is broad, cheap "does this crash the lexer/parser"
/// coverage, not a sophisticated search; see
/// docs/adr/ADR-0014-fuzz-harness-implementation.md.
pub fn mutate(bytes: &mut Vec<u8>, rng: &mut Rng) {
    if bytes.is_empty() {
        bytes.push(rng.next_u8());
        return;
    }
    if bytes.len() >= MAX_LEN {
        // At the cap - only the two non-growing ops are offered, so
        // repeated mutation can never make `bytes` grow without bound.
        if rng.next_usize(2) == 0 {
            let i = rng.next_usize(bytes.len());
            bytes[i] = rng.next_u8();
        } else {
            let i = rng.next_usize(bytes.len());
            bytes.remove(i);
        }
        return;
    }
    match rng.next_usize(4) {
        0 => {
            let i = rng.next_usize(bytes.len());
            bytes[i] = rng.next_u8();
        }
        1 => {
            let i = rng.next_usize(bytes.len() + 1);
            bytes.insert(i, rng.next_u8());
        }
        2 => {
            let i = rng.next_usize(bytes.len());
            bytes.remove(i);
        }
        _ => {
            let i = rng.next_usize(bytes.len());
            let available = bytes.len() - i;
            let budget = (MAX_LEN - bytes.len()).max(1);
            let len = rng.next_usize(available.min(budget)).max(1).min(available);
            let dup: Vec<u8> = bytes[i..i + len].to_vec();
            let at = rng.next_usize(bytes.len() + 1);
            for (offset, b) in dup.into_iter().enumerate() {
                bytes.insert(at + offset, b);
            }
        }
    }
}

/// A small seed corpus: the six canonical example contracts (this
/// project's own conformance corpus, per `examples/canonical/`) plus a
/// handful of hand-picked edge cases chosen to stress lexer/parser
/// boundary conditions golden-file testing doesn't specifically target
/// (empty input, a lone multi-byte UTF-8 character, deeply nested
/// delimiters with no closer, an unterminated string, a NUL byte).
pub fn seed_corpus() -> Vec<String> {
    let mut seeds: Vec<String> = vec![
        include_str!("../../../examples/canonical/counter.kyn").to_string(),
        include_str!("../../../examples/canonical/token.kyn").to_string(),
        include_str!("../../../examples/canonical/escrow.kyn").to_string(),
        include_str!("../../../examples/canonical/marketplace.kyn").to_string(),
        include_str!("../../../examples/canonical/voting.kyn").to_string(),
        include_str!("../../../examples/canonical/multisig_wallet.kyn").to_string(),
    ];
    seeds.extend([
        String::new(),
        "\u{1F600}".to_string(),
        "(".repeat(1000),
        "\"unterminated string".to_string(),
        "contract".to_string(),
        "0".repeat(500),
        "//".to_string(),
        "\0".to_string(),
        "match {} => {}".to_string(),
    ]);
    seeds
}

/// Runs `target` against `iterations` mutated variants drawn from
/// [`seed_corpus`], seeded from `seed` for reproducibility. Each
/// mutated byte string is coerced to valid UTF-8 via
/// [`String::from_utf8_lossy`] before being handed to `target`, since
/// both `kyne_lexer::lex` and `kyne_parser::parse` take `&str` - this
/// still exercises adversarial, non-source-like input (`from_utf8_lossy`
/// substitutes U+FFFD for invalid sequences rather than producing
/// "clean" text), including multi-byte character boundary edge cases.
///
/// Panics with the exact input that crashed if `target` ever panics -
/// per this crate's whole purpose, that panic is the test failure this
/// harness exists to catch, so it is deliberately not swallowed.
pub fn run_fuzz_loop(seed: u64, iterations: usize, target: impl Fn(&str)) {
    let corpus = seed_corpus();
    let mut rng = Rng::new(seed);
    for i in 0..iterations {
        let base = &corpus[rng.next_usize(corpus.len())];
        let mut bytes = base.clone().into_bytes();
        let mutation_count = 1 + rng.next_usize(8);
        for _ in 0..mutation_count {
            mutate(&mut bytes, &mut rng);
        }
        let text = String::from_utf8_lossy(&bytes).into_owned();

        let result = panic::catch_unwind(AssertUnwindSafe(|| target(&text)));
        if let Err(payload) = result {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic payload>".to_string());
            panic!(
                "fuzz iteration {i} (seed {seed:#x}) panicked: {message}\ninput ({} bytes): {text:?}",
                text.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_the_same_mutated_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn mutate_never_panics_on_repeated_application() {
        let mut rng = Rng::new(1);
        let mut bytes = Vec::new();
        for _ in 0..1000 {
            mutate(&mut bytes, &mut rng);
        }
    }

    #[test]
    fn run_fuzz_loop_catches_a_real_panic() {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            run_fuzz_loop(1, 10, |text| {
                if text.len() > 3 {
                    panic!("deliberate test panic");
                }
            });
        }));
        assert!(
            result.is_err(),
            "expected run_fuzz_loop to propagate the target's panic"
        );
    }

    #[test]
    fn seed_corpus_is_nonempty_and_includes_the_canonical_examples() {
        let corpus = seed_corpus();
        assert!(corpus.len() >= 6);
        assert!(corpus.iter().any(|s| s.contains("contract Counter")));
    }
}
