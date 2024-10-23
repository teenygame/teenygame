# teenygame

**teenygame** is a real simple multiplatform game framework for Rust.

It's designed for 2D games with a focus on providing a way to draw graphics, play audio, and handle input. That's it!

## Features
- 2D graphics!
- Touch events!
- Audio!

## Supported platforms

- 🟢 **Linux**
- 🟢 **macOS**
- 🟢 **Windows**
- 🟢 **Web**
- ⚠️ **iOS:** Gets stuck after rendering first frame. No support for app lifecycle (e.g. suspend).
- ❓ **Android:** Untested.

### Web

You can run games in your browser using `wasm-server-runner`:

First install it:

```sh
cargo install wasm-server-runner
```

Then set in up in your `.cargo/config.toml` in either your project or home folder:

```toml
[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```

You can now run your game like any other Rust binary target:

```sh
cargo run --target wasm32-unknown-unknown
```

## Examples

- [Snake](examples/snake): A simple Snake game.
- [Bullet Hell](examples/bullet-hell): A bullet hell game.
