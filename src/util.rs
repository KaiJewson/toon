use std::fmt;

pub(crate) struct WriteFn<F: FnMut(&str)>(pub(crate) F);

impl<F: FnMut(&str)> fmt::Write for WriteFn<F> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0(s);
        Ok(())
    }
}

pub(crate) struct WriteCharsFn<F: FnMut(char)>(pub(crate) F);

impl<F: FnMut(char)> fmt::Write for WriteCharsFn<F> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.0(c);
        }
        Ok(())
    }
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0(c);
        Ok(())
    }
}
