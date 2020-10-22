//! Backends for Toon.

use std::fs::{self, File};
use std::future::Future;
use std::io::{self, BufWriter, IoSlice, Write};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, RawHandle};

use stdio_override::{StdoutOverride, StderrOverride};
use os_pipe::PipeReader;

use crate::{Color, CursorShape, Input, Intensity, Vec2};

#[cfg(feature = "crossterm")]
mod crossterm;
mod dummy;

#[cfg(feature = "crossterm")]
pub use self::crossterm::Crossterm;
pub use self::dummy::*;

/// A backend that can be used with Toon.
pub trait Backend {
    /// Errors produced by this backend.
    type Error;
    /// The backend when bound to a TTY.
    type Bound: Bound<Error = Self::Error>;

    /// Attempt to bind the backend to a TTY.
    ///
    /// # Errors
    ///
    /// Fails if initializing the backend fails.
    fn bind(self, io: Tty) -> Result<Self::Bound, Self::Error>;

    /// Whether the backend is a dummy backend that does not need real access to a TTY.
    ///
    /// Default is `false`.
    #[must_use]
    fn is_dummy() -> bool {
        false
    }
}

/// A backend bound to a TTY.
///
/// Operations should be buffered and `flush` should flush them. Since `Tty` uses a `BufWriter`
/// internally this will often not have to be done manually.
#[allow(clippy::missing_errors_doc)]
pub trait Bound: for<'a> ReadEvents<'a, EventError = <Self as Bound>::Error> + Sized {
    /// Error executing an operation.
    type Error;

    // General functions

    /// Get the size of the terminal.
    fn size(&mut self) -> Result<Vec2<u16>, Self::Error>;

    /// Set the title of the terminal.
    fn set_title(&mut self, title: &str) -> Result<(), Self::Error>;

    // Cursor functions

    /// Hide the cursor.
    fn hide_cursor(&mut self) -> Result<(), Self::Error>;

    /// Show the cursor.
    fn show_cursor(&mut self) -> Result<(), Self::Error>;

    /// Set the cursor shape.
    fn set_cursor_shape(&mut self, shape: CursorShape) -> Result<(), Self::Error>;

    /// Set whether the cursor blinks.
    fn set_cursor_blinking(&mut self, blinking: bool) -> Result<(), Self::Error>;

    /// Set the position of the cursor (zero-indexed).
    fn set_cursor_pos(&mut self, pos: Vec2<u16>) -> Result<(), Self::Error>;

    // Style functions

    /// Set the foreground color to write with.
    fn set_foreground(&mut self, foreground: Color) -> Result<(), Self::Error>;

    /// Set the background color to write with.
    fn set_background(&mut self, background: Color) -> Result<(), Self::Error>;

    /// Set the text intensity.
    fn set_intensity(&mut self, intensity: Intensity) -> Result<(), Self::Error>;

    /// Set whether the text is emphasized.
    fn set_italic(&mut self, italic: bool) -> Result<(), Self::Error>;

    /// Set whether the text is underlined.
    fn set_underlined(&mut self, underlined: bool) -> Result<(), Self::Error>;

    /// Set whether the text blinks.
    fn set_blinking(&mut self, blinking: bool) -> Result<(), Self::Error>;

    /// Set whether the text is crossed out.
    fn set_crossed_out(&mut self, crossed_out: bool) -> Result<(), Self::Error>;

    // Writing

    /// Write text to the output.
    ///
    /// This text is guaranteed not to contain control characters. Writing text will never cause
    /// the line to overflow or wrap.
    fn write(&mut self, text: &str) -> Result<(), Self::Error>;

    // Finalizing functions

    /// Flush all buffered actions to the tty.
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Reset the terminal to its initial state, returning the TTY.
    ///
    /// This will always be called.
    fn reset(self) -> Result<Tty, Self::Error>;
}

/// Backends which can read events.
pub trait ReadEvents<'a> {
    /// This error type must be the same type as used in `Bound`.
    type EventError;

    /// The future that reads the next input value.
    ///
    /// Dropping this future must stop reading input.
    type EventFuture: Future<Output = Result<TerminalEvent, Self::EventError>> + 'a;

    /// Read the next event from the terminal.
    fn read_event(&'a mut self) -> Self::EventFuture;
}

/// An event on the terminal.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TerminalEvent {
    /// A user input occurred.
    ///
    /// If it is a mouse input, the `size` field is ignored, so you can fill it with anything.
    Input(Input),
    /// The terminal was resized. Contains the new size.
    Resize(Vec2<u16>),
}

/// A type which backends use to perform I/O.
///
/// Internally it uses a `BufWriter` so all write calls are buffered. If you are using both the
/// `Write` impl and the `AsRawFd`/`AsRawHandle` impl take care to flush it, otherwise you'll get
/// inconsistencies.
#[derive(Debug)]
pub struct Tty {
    inner: Option<BufWriter<TtyInner>>,
}

impl Tty {
    pub(crate) fn dummy() -> Self {
        Self {
            inner: None
        }
    }
    pub(crate) fn new() -> io::Result<(Self, PipeReader)> {
        let (inner, writer) = TtyInner::new()?;
        Ok((Self { inner: Some(BufWriter::new(inner)), }, writer))
    }
    pub(crate) fn cleanup(self) -> io::Result<()> {
        if let Some(inner) = self.inner {
            inner.into_inner()?.cleanup()?;
        }
        Ok(())
    }
}

impl Write for Tty {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.as_mut().unwrap().write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.inner.as_mut().unwrap().write_vectored(bufs)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.as_mut().unwrap().flush()
    }
}

#[cfg(unix)]
impl AsRawFd for Tty {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_ref().unwrap().get_ref().as_raw_fd()
    }
}
#[cfg(windows)]
impl AsRawHandle for Tty {
    fn as_raw_handle(&self) -> RawHandle {
        self.inner.as_ref().unwrap().get_ref().as_raw_handle()
    }
}

#[derive(Debug)]
struct TtyInner {
    stdout: StdoutOverride,
    stderr: StderrOverride,
    tty: Option<File>,
}

impl TtyInner {
    fn new() -> io::Result<(Self, PipeReader)> {
        let (rx, tx) = os_pipe::pipe()?;

        let stdout = StdoutOverride::from_io_ref(&tx)?;
        let stderr = StderrOverride::from_io(tx)?;

        let tty = if cfg!(unix) {
            let tty_path = if cfg!(target_os = "redox") {
                std::env::var("TTY").ok()
            } else {
                Some("/dev/tty".to_owned())
            };

            tty_path
                .and_then(|path| {
                    fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(path)
                        .ok()
                })
        } else {
            None
        };

        Ok((
            Self {
                stdout,
                stderr,
                tty,
            },
            rx,
        ))
    }
    fn cleanup(self) -> io::Result<()> {
        self.stdout.reset()?;
        self.stderr.reset()?;
        Ok(())
    }
}

#[allow(clippy::option_if_let_else)]
impl Write for TtyInner {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(tty) = &mut self.tty {
            tty.write(buf)
        } else {
            self.stdout.write(buf)
        }
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        if let Some(tty) = &mut self.tty {
            tty.write_vectored(bufs)
        } else {
            self.stdout.write_vectored(bufs)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(tty) = &mut self.tty {
            tty.flush()
        } else {
            self.stdout.flush()
        }
    }
}

#[cfg(unix)]
impl AsRawFd for TtyInner {
    fn as_raw_fd(&self) -> RawFd {
        self.tty.as_ref().map_or_else(|| self.stdout.as_raw_fd(), |tty| tty.as_raw_fd())
    }
}
#[cfg(windows)]
impl AsRawHandle for TtyInner {
    fn as_raw_handle(&self) -> RawHandle {
        // Windows doesn't have /dev/tty
        self.stdout.as_raw_handle()
    }
}
