pub fn assert_eq_approx<T>(a: T, b: T, tolerance: T)
where
    T: PartialOrd + std::ops::Sub<Output = T> + Copy + std::fmt::Debug + Abs,
{
    if (a - b).abs() > tolerance {
        panic!(
            "assertion failed: `(left ~= right)`\n  left: `{:?}`,\n right: `{:?}`",
            a, b
        );
    }
}

pub trait Abs {
    fn abs(self) -> Self;
}

impl Abs for f32 {
    fn abs(self) -> Self {
        if self < 0.0 {
            -self
        } else {
            self
        }
    }
}

impl Abs for f64 {
    fn abs(self) -> Self {
        if self < 0.0 {
            -self
        } else {
            self
        }
    }
}

impl Abs for i16 {
    fn abs(self) -> Self {
        if self < 0 {
            -self
        } else {
            self
        }
    }
}

impl Abs for i32 {
    fn abs(self) -> Self {
        if self < 0 {
            -self
        } else {
            self
        }
    }
}

impl Abs for i64 {
    fn abs(self) -> Self {
        if self < 0 {
            -self
        } else {
            self
        }
    }
}
