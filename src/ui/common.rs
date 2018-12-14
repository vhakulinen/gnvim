
pub fn calc_line_space(space: i64) -> (i32, i32) {
    let half = space as f64 / 2.0;
    if half as f64 % 2.0 != 0.0 {
        (half.ceil() as i32, half.floor() as i32)
    } else {
        (half as i32, half as i32)
    }
}

#[test]
fn test_calc_line_space() {
    assert_eq!((1, 0), calc_line_space(1));
    assert_eq!((1, 1), calc_line_space(2));
    assert_eq!((3, 2), calc_line_space(5));
}
