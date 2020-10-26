//! A simple hello world example in Toon.

use toon::{Crossterm, ElementExt, Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start the async runtime. We're using smol here, but you can use any runtime you like.
    smol::block_on(async {
        // Initialize the terminal with the Crossterm backend.
        let mut terminal = Terminal::new(Crossterm::default())?;

        // Draw `Hello World` on the terminal, configure a unit event to trigger when the q key is
        // pressed, and wait for the next event.
        terminal
            .draw(toon::span("Hello World!").on('q', |_| ()))
            .await?;

        // Clean up the terminal. This is not strictly necessary as it's also called in the
        // terminal's destructor, but errors won't be ignored this way.
        terminal.cleanup()?;

        Ok(())
    })
}
