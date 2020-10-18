//! A simple hello world example in Toon.

use toon::{Crossterm, CrosstermConfig, ElementExt, Style, Terminal};

fn main() {
    let res = smol::block_on(async {
        let mut terminal: Terminal<Crossterm> = Terminal::new(CrosstermConfig::default())?;

        terminal
            .draw(toon::text("Hello World!", Style::default()).on('q', ()))
            .await?;

        terminal.cleanup()
    });

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }
}
