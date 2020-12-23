# toon

[![Github](https://img.shields.io/badge/repository-github-24292e)](https://github.com/KaiJewson/toon)
[![Crates.io](https://img.shields.io/crates/v/toon)](https://crates.io/crates/toon)
[![docs.rs](https://docs.rs/toon/badge.svg)](https://docs.rs/toon)

A simple, declarative, and modular TUI library.

In Toon, every application starts out with some **state**. Then, using your state you create an
**element** (the [`Element`](https://docs.rs/toon/0.1/toon/trait.Element.html) trait). You pass
your element to Toon using
[`Terminal::draw`](https://docs.rs/toon/0.1/toon/struct.Terminal.html#method.draw) and it
renders it to the screen, before waiting for user input. When that occurs, Toon uses your
element to translate it into some number of **events**, which are then used to modify your
state, and the cycle repeats.

```
         Drawing                Input
State ────────────→ Elements ──────────→ Events
  ↑                                        │
  ╰────────────────────────────────────────╯
```

As such, your UI is a simple pure function of your state. This helps eliminate a whole class of
inconsistency bugs; given a certain state, your UI will look the exact same way, _always_. The
event system also allows you to easily trace each and every modification to your state, which
can be very useful.

See the [comparison](https://github.com/KaiJewson/toon/blob/master/COMPARISON.md) to compare it
with the other big TUI libraries, [Cursive](https://github.com/gyscos/cursive) and
[tui](https://github.com/fdehau/tui-rs).

## Example

See the [examples](https://github.com/KaiJewson/toon/tree/master/examples) folder for more.

Display `Hello World!` on the terminal using the Crossterm backend:
```rust
use toon::{Crossterm, Terminal, ElementExt};

let mut terminal = Terminal::new(Crossterm::default())?;

terminal
    .draw(toon::span("Hello World!").on('q', |_| ()))
    .await?;

terminal.cleanup()
```

## Features

Toon offers the following features, none of which are enabled by default:
- `crossterm`: Enable the
[Crossterm](https://docs.rs/toon/0.1/toon/backend/struct.Crossterm.html) backend.
- `dev`: Enable developer tools.
- `either`: Integrate with the [`either`](https://crates.io/crates/either) crate. This
implements [`Element`](https://docs.rs/toon/0.1/toon/trait.Element.html),
[`Output`](https://docs.rs/toon/0.1/toon/output/trait.Output.html) and
[`Collection`](https://docs.rs/toon/0.1/toon/elements/containers/trait.Collection.html) for
`Either`.

License: MIT OR Apache-2.0
