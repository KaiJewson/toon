//! A program that displays all the terminal's events, for experimenting and testing purposes.

use toon::{Crossterm, ElementExt, Input, ScrollOffset, Terminal};

enum Event {
    Quit,
    Input(Input),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let mut terminal = Terminal::new(Crossterm::default())?;

        let mut elements = Vec::new();

        'outer: loop {
            let events = terminal
                .draw(
                    toon::column(toon::flow(), &elements)
                        .scroll_y(ScrollOffset::End(0))
                        .on(|_| true, Event::Input)
                        .on('q', |_| Event::Quit),
                )
                .await?;

            for event in events {
                match event {
                    Event::Quit => break 'outer,
                    Event::Input(input) => elements.push(toon::span(format!("{:?}", input))),
                }
            }
        }

        terminal.cleanup()?;
        Ok(())
    })
}
