//! An example using Toon's developer tools functionality.

use smol::future;
use smol::stream::StreamExt;
use toon::{dev, Crossterm, ElementExt, Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let mut terminal = Terminal::new(Crossterm::default())?;

        // Initialize the dev state
        let mut dev = toon::Dev::new();
        // Get the stream of captured output.
        let mut dev_events = dev::display_captured(terminal.take_captured().unwrap());

        'outer: loop {
            // Wrap the element in dev tools
            let element = dev.wrap(
                toon::span("Hello World!")
                    .title("Hello World!")
                    .on('q', |_| ()),
            );

            let events = future::race(terminal.draw(element), async {
                Ok(vec![dev::AppEvent::Dev(dev_events.next().await.unwrap())])
            })
            .await?;

            for event in events {
                match event {
                    // An event in the element itself telling us to quit.
                    dev::AppEvent::Element(()) => break 'outer,
                    // An event in the dev tools, which we then apply ot its state.
                    dev::AppEvent::Dev(e) => dev.apply(e),
                }
            }
        }

        terminal.cleanup()?;
        Ok(())
    })
}
