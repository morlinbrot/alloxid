# alloxid-front
Frontend of the `alloxid` family of crates made with [Leptos](https://github.com/leptos-rs/leptos).

The app expects `alloxid-http` to be running at `localhost:3000`.

## Usage
Run with [cargo-leptos](https://github.com/leptos-rs/cargo-leptos):
```
cargo leptos watch
```

Package with:
```
cargo leptos build
```

## Leptos: Installing Additional Tools

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
2. `rustup default nightly` - setup nightly as default, or you can use rust-toolchain file later on
3. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
4. `cargo install cargo-generate` - install `cargo-generate` binary (should be installed automatically in future)
5. `npm install -g sass` - install `dart-sass` (should be optional in future
