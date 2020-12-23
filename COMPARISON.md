# Comparison

This document compares Toon with the two existing main Rust TUI libraries,
[Cursive](https://github.com/gyscos/cursive) and [tui](https://github.com/fdehau/tui-rs).

## Terminology

First, the libraries have some terminology differences, which are important to know about.

| Description | Toon | Cursive | tui-rs |
| --- | --- | --- | --- |
| Primary type for using the library | Terminal | Cursive | Terminal |
| A composable part of the UI | Element | View | Widget |
| A coordinate | Vec2 | XY and Vec2 | Tuple |
| Type to which UI parts draw themselves | Output | Printer | Buffer |
| User inputs | Input | Event | N/A |

## Paradigm

Toon and tui are both declarative and immediate; you create a `Terminal` type, and in your main loop
every time your state changes you redraw the entire UI (they both employ diffing algorithms to send
the minimum number of commands to the terminal as possible). Cursive on the other hand is totally
different; you create a `Cursive`, set it up with all the UI that you want and then completely give
over control to it. Views can have their own internal state, and the UI is updated in parts rather
than all at once.

## Input

Tui does not handle user input at all, it's completely left up to the user. Toon and Cursive both
handle user input built-in, but they do it slightly differently; in Cursive user input directly
calls callbacks which can modify the UI's state, but in Toon user input is given to your element
which translates it into some number of events, and then you use the list of events to update your
state and redraw the UI. Toon also handles user input asynchronously, allowing you to run many
tasks at once.

## Cursors

Cursive does not support using the terminal's native cursor at all. Tui allows you to control the
cursor globally from your `Terminal` instance, but it only supports changing its visibility and its
position. In Toon every element can control the visibility, position, shape and blinking state of
its cursor which will often forward up to the native terminal cursor, but custom cursors can also be
set.

## Terminal Title

Cursive and tui do not support setting the terminal title. In Toon every element has a title, which
like with cursors will often forward up to the terminal title, but things like borders can also
intercept and use it.

## Output

Cursive and tui both use a concrete struct for outputs, called `Printer` and `Buffer`, and they
allow you to write to the terminal. Toon on the other hand uses a trait for this purpose, the
`Output` trait. This allows for features like filters to be efficiently implemented, as you don't
have to create a whole new buffer just to later filter its contents and write it to the actual
buffer, you can simply forward all calls.

## Backends

All three libraries support multiple backends. Cursive and Toon support it with a backend trait,
while tui has mutually exclusive feature-flags. Cursive supports bear lib terminal, ncurses,
pancurses, termion and crossterm; tui supports termion, rustbox, crossterm and pancurses; and Toon
currently only supports crossterm.
