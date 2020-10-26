//! A simple stopwatch.

use std::time::{Duration, Instant};

use smol::{future, Timer};
use toon::{Crossterm, ElementExt, Styled, Terminal};

/// The state of the stopwatch.
enum Stopwatch {
    /// The stopwatch is running. Contains the instant at which it started running.
    Running(Instant),
    /// The stopwatch is stopped. Contains the duration it was when it stopped.
    Stopped(Duration),
}

/// Events that can occur.
enum Event {
    /// Toggle the state of the stopwatch.
    Toggle,
    /// Reset the stopwatch.
    Reset,
    /// Quit.
    Quit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        // The stopwatch starts in a stopped state with 0 seconds.
        let mut stopwatch = Stopwatch::Stopped(Duration::default());

        let mut terminal = Terminal::new(Crossterm::default())?;

        'outer: loop {
            // Get the duration to display on the stopwatch.
            let duration = match stopwatch {
                Stopwatch::Running(since) => since.elapsed(),
                Stopwatch::Stopped(duration) => duration,
            };

            // Wait for the first of two events:
            let events = future::race(
                // - The user causing some events to occur.
                terminal.draw(
                    toon::column((
                        toon::span(format_args!(
                            "{}:{:03}",
                            duration.as_secs(),
                            duration.subsec_millis()
                        ))
                        .bold()
                        .float((toon::Alignment::Middle, toon::Alignment::Middle)),
                        toon::span("[Space] Start/Stop      [R]: Reset           [Q]: Quit"),
                    ))
                    .on(' ', |_| Event::Toggle)
                    .on('r', |_| Event::Reset)
                    .on('q', |_| Event::Quit),
                ),
                // - The timer, while running, hasn't been updated in a while.
                async {
                    if let Stopwatch::Stopped(_) = stopwatch {
                        // Don't fire useless events if the timer is stopped.
                        future::pending::<()>().await;
                    } else {
                        Timer::after(Duration::from_millis(15)).await;
                    }
                    Ok(Vec::new())
                },
            )
            .await?;

            for event in events {
                match event {
                    Event::Toggle => {
                        stopwatch = match stopwatch {
                            Stopwatch::Running(since) => Stopwatch::Stopped(since.elapsed()),
                            Stopwatch::Stopped(duration) => {
                                Stopwatch::Running(Instant::now() - duration)
                            }
                        }
                    }
                    Event::Reset => {
                        stopwatch = match stopwatch {
                            Stopwatch::Running(_) => Stopwatch::Running(Instant::now()),
                            Stopwatch::Stopped(_) => Stopwatch::Stopped(Duration::default()),
                        }
                    }
                    Event::Quit => break 'outer,
                }
            }
        }

        terminal.cleanup()?;
        Ok(())
    })
}
