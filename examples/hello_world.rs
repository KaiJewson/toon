//! A simple hello world example in Toon.

use toon::{
    Attributes, Color, Crossterm, CrosstermConfig, Style,
    Terminal, ElementExt,
};

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
                    toon::Text::new(format_args!("The number is {}!", counter), Style::new(
                        Color::Red,
                        Color::Black,
                        Attributes::new().bold().italic().underlined().crossed_out(),
                    ))
                    .on(' ', Event::Increment)
                    .on('q', Event::Quit)
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
