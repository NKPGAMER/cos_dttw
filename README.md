# Double Tap to Wake & AOD Helper Daemon
> Target: **Xiaomi 15T Pro** (`klimt` – ColorOS Port)

A lightweight daemon designed to handle double-tap to wake and Always-On Display (AOD) interactions by communicating directly with the vendor touch interface.

---

## Credits

* **Original Logic & Discovery**: [Artic15th](https://t.me/ArticPorts) — Reverse-engineered the `ITouchFeature` AIDL interface and developed the initial Magisk module.

---

## Build Instructions

### Prerequisites

Ensure you have [Rust](https://www.rust-lang.org/) installed via `rustup` along with the `aarch64-unknown-linux-musl` target:

```bash
rustup target add aarch64-unknown-linux-musl

```

### Compilation

Compile with size optimizations and static linking using `lld`:

```bash
rustc \
  --target aarch64-unknown-linux-musl \
  -C linker="${HOME}/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld" \
  -C opt-level=z \
  -C panic=abort \
  -C strip=symbols \
  main.rs \
  -o android.hardware.touch@2.1-service.mediatek

```

> **Note**: Adjust the toolchain path in the `-C linker` flag if your local Rust installation directory differs.
