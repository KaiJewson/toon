use std::mem;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// A 2-dimensional vector.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Vec2<T> {
    /// The x component on the horizontal axis.
    pub x: T,
    /// The y component on the vertical axis.
    pub y: T,
}

impl<T> Vec2<T> {
    /// Create a new vector.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Swaps the x and y components.
    #[must_use]
    pub fn swap(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }

    /// Swaps the x and y components in place.
    pub fn swapped(&mut self) {
        mem::swap(&mut self.x, &mut self.y);
    }

    /// Map both the x and y components.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Vec2<U> {
        Vec2 {
            x: f(self.x),
            y: f(self.y),
        }
    }

    /// Convert the inner type of the vector.
    pub fn into<U>(self) -> Vec2<U>
    where
        T: Into<U>,
    {
        Vec2 {
            x: self.x.into(),
            y: self.y.into(),
        }
    }

    /// Zip this vector with another, producing a vector of a tuples.
    #[must_use]
    pub fn zip<U>(self, other: Vec2<U>) -> Vec2<(T, U)> {
        Vec2 {
            x: (self.x, other.x),
            y: (self.y, other.y),
        }
    }

    /// Get references to each of the components.
    #[must_use]
    pub fn as_ref(&self) -> Vec2<&T> {
        Vec2::new(&self.x, &self.y)
    }

    /// Get mutable references to each of the components.
    #[must_use]
    pub fn as_mut(&mut self) -> Vec2<&mut T> {
        Vec2::new(&mut self.x, &mut self.y)
    }
}

impl<T> Vec2<Option<T>> {
    /// Get a vector of the two components if they are both `Some`.
    #[must_use]
    pub fn both_some(self) -> Option<Vec2<T>> {
        Some(Vec2 {
            x: self.x?,
            y: self.y?,
        })
    }
}

impl<T: Add> Vec2<T> {
    /// Get the sum of the x and y components of the vector.
    pub fn sum(self) -> <T as Add>::Output {
        self.x + self.y
    }
}
impl<T: Mul> Vec2<T> {
    /// Get the product of the x and y components of the vector.
    pub fn product(self) -> <T as Mul>::Output {
        self.x * self.y
    }
}

impl<T: Ord> Vec2<T> {
    /// Computes the minimum of the two vectors in both dimensions.
    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    /// Computes the maximum of the two vectors in both dimensions.
    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    /// Computes the minimum and the maximum of the two vectors in both dimensions.
    pub fn min_max(self, other: Self) -> (Self, Self) {
        let (min_x, max_x) = if self.x <= other.x {
            (self.x, other.x)
        } else {
            (other.x, self.x)
        };
        let (min_y, max_y) = if self.y <= other.y {
            (self.y, other.y)
        } else {
            (other.y, self.y)
        };
        (Self::new(min_x, min_y), Self::new(max_x, max_y))
    }
}

macro_rules! vec2_arith {
    ($name:ident, $name_assign:ident, $method:ident, $method_assign:ident) => {
        impl<T: $name> $name for Vec2<T> {
            type Output = Vec2<<T as $name>::Output>;

            fn $method(self, rhs: Self) -> Self::Output {
                Vec2 {
                    x: <T as $name>::$method(self.x, rhs.x),
                    y: <T as $name>::$method(self.y, rhs.y),
                }
            }
        }

        impl<'a, T: $name<&'a T>> $name<&'a Vec2<T>> for Vec2<T> {
            type Output = Vec2<<T as $name<&'a T>>::Output>;

            fn $method(self, rhs: &'a Vec2<T>) -> Self::Output {
                Vec2 {
                    x: <T as $name<&'a T>>::$method(self.x, &rhs.x),
                    y: <T as $name<&'a T>>::$method(self.y, &rhs.y),
                }
            }
        }

        impl<'a, T> $name<Vec2<T>> for &'a Vec2<T>
        where
            &'a T: $name<T>,
        {
            type Output = Vec2<<&'a T as $name<T>>::Output>;

            fn $method(self, rhs: Vec2<T>) -> Self::Output {
                Vec2 {
                    x: <&'a T as $name<T>>::$method(&self.x, rhs.x),
                    y: <&'a T as $name<T>>::$method(&self.y, rhs.y),
                }
            }
        }

        impl<'a, 'b, T> $name<&'b Vec2<T>> for &'a Vec2<T>
        where
            &'a T: $name<&'b T>,
        {
            type Output = Vec2<<&'a T as $name<&'b T>>::Output>;

            fn $method(self, rhs: &'b Vec2<T>) -> Self::Output {
                Vec2 {
                    x: <&'a T as $name<&'b T>>::$method(&self.x, &rhs.x),
                    y: <&'a T as $name<&'b T>>::$method(&self.y, &rhs.y),
                }
            }
        }

        impl<T: $name_assign> $name_assign for Vec2<T> {
            fn $method_assign(&mut self, rhs: Self) {
                <T as $name_assign>::$method_assign(&mut self.x, rhs.x);
                <T as $name_assign>::$method_assign(&mut self.y, rhs.y);
            }
        }

        impl<'a, T: $name_assign<&'a T>> $name_assign<&'a Vec2<T>> for Vec2<T> {
            fn $method_assign(&mut self, rhs: &'a Self) {
                <T as $name_assign<&'a T>>::$method_assign(&mut self.x, &rhs.x);
                <T as $name_assign<&'a T>>::$method_assign(&mut self.y, &rhs.y);
            }
        }
    };
}

vec2_arith!(Add, AddAssign, add, add_assign);
vec2_arith!(Sub, SubAssign, sub, sub_assign);

impl<T: Neg> Neg for Vec2<T> {
    type Output = Vec2<<T as Neg>::Output>;

    fn neg(self) -> Self::Output {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> From<(T, T)> for Vec2<T> {
    fn from(xy: (T, T)) -> Self {
        Self { x: xy.0, y: xy.1 }
    }
}

impl<T> From<Vec2<T>> for (T, T) {
    fn from(vec: Vec2<T>) -> Self {
        (vec.x, vec.y)
    }
}

impl<T> From<Vec2<T>> for [T; 2] {
    fn from(vec: Vec2<T>) -> Self {
        [vec.x, vec.y]
    }
}

#[cfg(test)]
#[test]
fn vec_test() {
    let mut vec = Vec2::new(5, 6);
    assert_eq!(vec.sum(), 11);
    assert_eq!(vec.product(), 30);

    assert_eq!(vec.swap(), Vec2::new(6, 5));
    vec.swapped();
    assert_eq!(vec, Vec2::new(6, 5));

    let mut other = Vec2::new(2, 7);

    assert_eq!(vec.min(other), Vec2::new(2, 5));
    assert_eq!(vec.max(other), Vec2::new(6, 7));
    assert_eq!(vec.min_max(other), (vec.min(other), vec.max(other)));

    assert_eq!(vec + other, Vec2::new(8, 12));
    other += vec;
    assert_eq!(other, Vec2::new(8, 12));
}
