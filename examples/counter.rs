//! A counter that can be incremented by pressing space.

use toon::{Attributes, Color, Crossterm, CrosstermConfig, ElementExt, Style, Terminal};

#[derive(Clone, Copy)]
enum Event {
    Increment,
    Quit,
}

fn main() {
    let res = smol::block_on(async {
        let mut terminal: Terminal<Crossterm> = Terminal::new(CrosstermConfig::default())?;

        let mut counter: usize = 0;

        'outer: loop {
            let events = terminal
                .draw(
                    toon::text(
                        format_args!("The number is {}!", counter),
                        Style::new(Color::Red, Color::Black, Attributes::new().bold()),
                    )
                    .on(' ', Event::Increment)
                    .on('q', Event::Quit),
                )
                .await?;

            for event in events {
                match event {
                    Event::Increment => counter += 1,
                    Event::Quit => break 'outer,
                }
            }
        }

        terminal.cleanup()
    });

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }
}
