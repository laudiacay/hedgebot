Setup instructions for `cargo-fuzz` found here: 

https://rust-fuzz.github.io/book/cargo-fuzz/setup.html

Run `cargo install cargo-fuzz`.

Fuzz directory initialized with `cargo fuzz init`. To add a new fuzz target,
create a new file under `fuzz/fuzz_targets/` and add the path to `fuzz/Cargo.toml`.

Use `cargo fuzz list` to list fuzzing targets.

You also need a nightly compiler to run `cargo-fuzz`: `rustup install nightly`

Run the fuzzer with: `cargo +nightly fuzz run <target name>`. Need to check if
there is a way to set `+nightly` as default without using for the main target.
