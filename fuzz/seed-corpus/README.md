# Fuzz Seed Corpus

This directory stores committed fuzz seeds as hexadecimal text so review diffs
stay readable. Live `cargo fuzz` corpus and crash artifacts remain ignored under
`fuzz/corpus/` and `fuzz/artifacts/`.

Validate the committed seeds:

```bash
scripts/materialize_fuzz_seeds.py --check
```

Materialize them into cargo-fuzz corpus directories before a local fuzz run:

```bash
scripts/materialize_fuzz_seeds.py
```

Seed directories must match fuzz binary names from `fuzz/Cargo.toml`.
