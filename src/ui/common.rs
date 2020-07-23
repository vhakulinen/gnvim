use futures::future::Future;

pub fn spawn_local<F: Future<Output = ()> + 'static>(f: F) {
    let c = glib::MainContext::default();
    c.spawn_local(f);
}

pub fn calc_line_space(space: i64) -> (i32, i32) {
    let half = space as f64 / 2.0;
    if half as f64 % 2.0 != 0.0 {
        (half.ceil() as i32, half.floor() as i32)
    } else {
        (half as i32, half as i32)
    }
}

/// Calculate the preferred width and x-position.
pub fn get_preferred_horizontal_position(
    area: &gdk::Rectangle,
    pos: &gdk::Rectangle,
    mut width: i32,
) -> (i32, i32) {
    let mut x = pos.x;

    let rigth = x + width;
    // If we're overflowing to the right...
    if rigth > area.width {
        let overflow = rigth - area.width;
        // Move our x position to the left, but not father that 0.
        x = (x - overflow).max(0);

        // And set our width to be either the original width, or truncate
        // it to area.width it happens to be smaller (otherwise we'd still
        // overflow).
        width = width.min(area.width);
    }

    (x, width)
}

/// Calculate the preferred height and y-position.
pub fn get_preferred_vertical_position(
    area: &gdk::Rectangle,
    pos: &gdk::Rectangle,
    mut height: i32,
) -> (i32, i32) {
    let mut y = pos.y - height;

    if y < area.y {
        let max_above = area.y + pos.y;
        let max_below = area.height - (pos.y + pos.height);

        if max_above > max_below {
            y = area.y;
            height = max_above;
        } else {
            y = pos.y + pos.height;
            height = height.min(max_below);
        }
    }

    (y, height)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_calc_line_space() {
        assert_eq!((1, 0), calc_line_space(1));
        assert_eq!((1, 1), calc_line_space(2));
        assert_eq!((3, 2), calc_line_space(5));
    }

    #[test]
    fn test_get_preferred_vertical_position1() {
        // Case 1: there is room just fine in the obvious position.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 300,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 30,
            width: 300,
            height: 15,
        };
        let height = 30;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 0);
        assert_eq!(h, 30);
    }

    #[test]
    fn test_get_preferred_vertical_position2() {
        // Case 2: there is no room above the `pos`, so we should position our
        // selves below the pos.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 300,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 15,
        };
        let height = 30;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 15);
        assert_eq!(h, 30);
    }

    #[test]
    fn test_get_preferred_vertical_position3() {
        // Case 3: there is no room above the `pos`, so we should position our
        // selves below the pos but in this case, we need to truncate our height too.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 35,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 15,
        };
        let height = 30;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 15);
        assert_eq!(h, 20);
    }

    #[test]
    fn test_get_preferred_vertical_position4() {
        // Case 4: there is no room above the `pos`, but below it there is even less
        // space. We should go above, but truncate our height.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 300,
            height: 40,
        };
        let pos = gdk::Rectangle {
            x: 0,
            y: 30,
            width: 300,
            height: 50,
        };
        let height = 80;
        let (y, h) = get_preferred_vertical_position(&area, &pos, height);
        assert_eq!(y, 0);
        assert_eq!(h, 30);
    }

    #[test]
    fn test_get_preferred_horizontal_position1() {
        // Case 1: Everything fits.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 10,
        };

        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 10;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 10);
    }

    #[test]
    fn test_get_preferred_horizontal_position2() {
        // Case 2: Width is trucated.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 5,
        };

        let pos = gdk::Rectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 10;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 5);
    }

    #[test]
    fn test_get_preferred_horizontal_position3() {
        // Case 3: X is moved to left.
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 20,
        };

        let pos = gdk::Rectangle {
            x: 15,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 15;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 5);
        assert_eq!(w, 15);
    }

    #[test]
    fn test_get_preferred_horizontal_position4() {
        // Case 4: X is moved to left and width is truncated
        let area = gdk::Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 20,
        };

        let pos = gdk::Rectangle {
            x: 15,
            y: 0,
            width: 0,
            height: 0,
        };

        let width = 150;
        let (x, w) = get_preferred_horizontal_position(&area, &pos, width);
        assert_eq!(x, 0);
        assert_eq!(w, 20);
    }
}
