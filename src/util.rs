use std::fmt;

pub(crate) struct WriteFn<F: FnMut(&str) -> fmt::Result>(pub(crate) F);

impl<F: FnMut(&str) -> fmt::Result> fmt::Write for WriteFn<F> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0(s)
    }
}

pub(crate) struct WriteCharsFn<F: FnMut(char) -> fmt::Result>(pub(crate) F);

impl<F: FnMut(char) -> fmt::Result> fmt::Write for WriteCharsFn<F> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.0(c)?;
        }
        Ok(())
    }
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0(c)
    }
}

// pub(crate) struct DisplayFn<F: Fn(&mut Formatter<'_>) -> fmt::Result>(pub(crate) F);
//
// impl<F: Fn(&mut Formatter<'_>) -> fmt::Result> Display for DisplayFn<F> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         (self.0)(f)
//     }
// }

pub(crate) enum Lazy<T, F> {
    Computed(T),
    Uncomputed(F),
}

impl<T: Default, F: FnOnce() -> T> Lazy<T, F> {
    pub(crate) fn new(f: F) -> Self {
        Self::Uncomputed(f)
    }

    pub(crate) fn get(&mut self) -> &mut T {
        match self {
            Self::Computed(val) => val,
            Self::Uncomputed(_) => {
                let func = match std::mem::replace(self, Self::Computed(T::default())) {
                    Self::Computed(_) => unreachable!(),
                    Self::Uncomputed(func) => func,
                };
                *self = Self::Computed(func());
                match self {
                    Self::Computed(val) => val,
                    Self::Uncomputed(_) => unreachable!(),
                }
            }
        }
    }
}
