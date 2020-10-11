//! A simple hello world example in Toon.

use std::fmt::Display;

use toon::{
    Attributes, Color, Crossterm, CrosstermConfig, CursorShape, Element, Input, Output, Style,
    Terminal, Vec2,
};

enum Event {
    Increment,
    Quit,
}

struct Text<T>(T);

impl<T: Display> Element<Event> for Text<T> {
    fn draw(&self, output: &mut dyn Output) {
        output.write(
            Vec2::new(0, 0),
            &self.0,
            Style::new(
                Color::Red,
                Color::Black,
                Attributes::new().bold().italic().underlined().crossed_out(),
            ),
        );
        output.set_title(&"uwu");
        output.set_cursor(Some(toon::Cursor {
            shape: CursorShape::Block,
            blinking: true,
            pos: Vec2::new(15, 30),
        }));
    }
    fn ideal_size(&self, maximum: Vec2<u16>) -> Vec2<u16> {
        maximum
    }
    fn handle(&self, input: Input) -> Option<Event> {
        Some(if input == 'q' {
            Event::Quit
        } else {
            Event::Increment
        })
    }
}

fn main() {
    let res = smol::block_on(async {
        let mut terminal: Terminal<Crossterm> = Terminal::new(CrosstermConfig::default())?;

        let mut counter: usize = 0;

        while let Event::Increment = terminal
            .draw(Text(format_args!("The number is {}!", counter)))
            .await?
        {
            counter += 1;
        }

        terminal.cleanup()
    });

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }
}
