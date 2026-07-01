# eth Toolchain Policy

`eth` pins stable Rust `1.96.1` in `rust-toolchain.toml` and supports Rust
`1.90.0` through `1.96.1`.

Rules:

- Update the pinned toolchain only after checking the latest stable Rust.
- Keep `workspace.package.rust-version` at the supported MSRV.
- Run compatibility checks for every supported local toolchain before releases.
- Do not require nightly for normal builds.
- Document any target-specific or future `no_std` exception before admission.

Compatibility command:

```bash
for toolchain in 1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1; do
    cargo "+$toolchain" check --workspace --all-features
done
```
