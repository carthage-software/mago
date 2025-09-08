# `mago-pager`

A Rust crate for piping output to a terminal pager (like `less` or `more`) on Unix-like systems.

This crate provides a simple builder-style API to configure and spawn a pager process, redirecting the standard output of your application to it. When the pager session ends, the original standard output is gracefully restored.

On non-Unix platforms (like Windows), this crate is a complete no-op, compiling successfully and allowing for seamless cross-platform development.

## A Hard Fork of `pager`

This crate is a hard fork of the unmaintained [`pager`](https://crates.io/crates/pager) crate. We created this fork to address several key issues and modernize the implementation.

The main differences are:

1.  **Uses a Sub-Process, Not a Fork**: The original `pager` crate used `fork()` to create the pager process. This had a significant drawback: if your application needed to exit with a non-zero status code (e.g., `exit(1)`), the `fork` model would cause it to adopt the pager's exit code instead (which is almost always `0`). This masks errors and breaks standard CLI behavior. `mago-pager` spawns a true **sub-process**, ensuring that your application's original exit code is always preserved.

2.  **Cross-Platform Compatibility**: The original `pager` crate fails to compile on Windows. `mago-pager` compiles on all platforms, acting as a zero-cost abstraction on non-Unix systems. This allows you to include it as a dependency without needing platform-specific `#[cfg]` attributes in your own code.

3.  **Modernized and Idiomatic API**: We have renamed many parts of the API to be more idiomatic and clear, improving the overall developer experience.

## Usage

Add `mago-pager` to your `Cargo.toml`:

```toml
[dependencies]
mago-pager = "*"
```

Then, use the `Pager` builder to spawn a session. The pager is active as long as the `PagerSession` variable is in scope.

```rust
use mago_pager::Pager;
use std::io::Result;

fn main() -> Result<()> {
    // Spawn a pager if the environment is suitable (Unix, TTY).
    // The session guard handles cleanup.
    let _session = Pager::new().spawn()?;

    // All subsequent writes to stdout will be piped to the pager.
    for i in 0..1000 {
        println!("This is line number {}", i);
    }

    // When `_session` goes out of scope at the end of `main`, the pager
    // is closed and the original stdout is restored.
    Ok(())
}
```

## Environment Variables

The pager's behavior can be controlled with the following environment variables:

- `MAGO_PAGER`: Overrides the default pager command. For example, `MAGO_PAGER="less -R"` will use `less` with color support.
- `NOPAGER`: If set to any value, the pager will be disabled entirely.

## License

This project is licensed under the MIT License.
