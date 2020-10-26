//! A counter that can be incremented by pressing space.

use toon::{Crossterm, ElementExt, Styled, Terminal};

/// Types of events that can happen in this application.
enum Event {
    /// The counter should be incremented.
    Increment,
    /// The application should exit.
    Quit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        // The counter to be incremented.
        let mut counter: usize = 0;

        let mut terminal = Terminal::new(Crossterm::default())?;

        'outer: loop {
            // Draw the UI and wait for events. Terminal::draw will return a Vec<Event>.
            let events = terminal
                .draw(
                    // toon::span takes in any type that implements Display, and format_args! is the
                    // cheapest way to display something.
                    toon::span(format_args!("The number is {}!", counter))
                        // Span implements AsRef<Style> and AsMut<Style> and so implements the
                        // Styled trait, which allows you to use these helpful methods.
                        .red()
                        .bold()
                        .on(' ', |_| Event::Increment)
                        .on('q', |_| Event::Quit),
                )
                .await?;

            // For every event that occurred...
            for event in events {
                // ...run the appropriate code to handle it.
                match event {
                    Event::Increment => counter += 1,
                    Event::Quit => break 'outer,
                }
            }
        }

        terminal.cleanup()?;
        Ok(())
    })
}
