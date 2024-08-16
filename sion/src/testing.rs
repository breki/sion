pub fn assert_eq_approx(a: f32, b: f32, tolerance: f32) {
    if (a - b).abs() > tolerance {
        panic!(
            "assertion failed: `(left ~= right)`\n  left: `{}`,\n right: `{}`",
            a, b
        );
    }
}
