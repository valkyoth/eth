# eth Unsafe Policy

First-party protocol-facing crates use:

```rust
#![forbid(unsafe_code)]
```

Unsafe Rust is not admitted in the current workspace.

If a future FFI, protected-memory, database, or performance boundary requires
unsafe code, it must be isolated in a dedicated crate with:

- a documented reason unsafe is necessary;
- a `SAFETY:` explanation for every unsafe block;
- tests that exercise the invariant;
- Miri or sanitizer coverage where applicable;
- security review before release.

Unsafe code must never be mixed with parser policy or protocol validation logic.
