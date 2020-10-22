//! A simple hello world example in Toon.

use toon::{Crossterm, ElementExt, Terminal};

fn main() {
    let res = smol::block_on(async {
        let mut terminal = Terminal::new(Crossterm::default())?;

        terminal
            .draw(toon::span("Hello World!").on('q', ()))
            .await?;

        terminal.cleanup()
    });

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }
}
