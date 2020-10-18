//! A simple hello world example in Toon.

use toon::{Crossterm, ElementExt, Style, Terminal};

fn main() {
    let res = smol::block_on(async {
        let mut terminal = Terminal::new(Crossterm::default())?;

        terminal
            .draw(toon::text("Hello World!", Style::default()).on('q', ()))
            .await?;

        terminal.cleanup()
    });

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }
}
