# toon

[![Crates.io](https://img.shields.io/crates/v/toon)](https://crates.io/crates/toon)
[![Github](https://img.shields.io/badge/-github-24292e)](https://github.com/KaiJewson/toon)
[![docs.rs](https://img.shields.io/badge/-docs.rs-informational)](https://docs.rs/toon)

A simple, declarative, and modular TUI library.

Get started by reading the
[tutorial](https://github.com/KaiJewson/toon/blob/master/TUTORIAL.md) and looking through the
[examples](https://github.com/KaiJewson/toon/tree/master/examples). See also the
[comparison](https://github.com/KaiJewson/toon/blob/master/COMPARISON.md) to compare it with
[tui](https://github.com/fdehau/tui-rs) and [Cursive](https://github.com/gyscos/cursive).

## Examples

Display `Hello World!` on the terminal using the Crossterm backend:

```rust
use toon::{Crossterm, Terminal, ElementExt};

let mut terminal = Terminal::new(Crossterm::default())?;

terminal
    .draw(toon::span("Hello World!").on('q', |_| ()))
    .await?;

terminal.cleanup()
```

License: MIT OR Apache-2.0
