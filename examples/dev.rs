//! An example using Toon's developer tools functionality.

use futures_lite::future;
use futures_lite::stream::StreamExt;
use toon::{dev, Crossterm, ElementExt, Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    future::block_on(async {
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

            let events = future::race(
                // The user has caused some events to occur.
                terminal.draw(element),
                // Something has been printed to the standard output or error and we need to update
                // the dev tools to display it.
                async { Ok(vec![dev::AppEvent::Dev(dev_events.next().await.unwrap())]) },
            )
            .await?;

            for event in events {
                match event {
                    // An event in the element itself telling us to quit.
                    dev::AppEvent::Element(()) => break 'outer,
                    // An event in the dev tools, which we then apply to its state.
                    dev::AppEvent::Dev(e) => dev.apply(e),
                }
            }
        }

        terminal.cleanup()?;
        Ok(())
    })
}
