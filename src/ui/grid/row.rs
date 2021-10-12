use crate::nvim_bridge::GridLineSegment;

#[cfg(test)]
use crate::nvim_bridge;

#[derive(Clone)]
pub struct Cell {
    pub text: String,
    pub hl_id: u64,
    pub double_width: bool,
}

#[derive(Debug, PartialEq)]
pub struct Segment {
    pub text: String,
    pub hl_id: u64,
    pub start: usize,
    pub len: usize,
}

/// Row, as in one row in a grid.
#[derive(Clone)]
pub struct Row {
    cells: Box<[Cell]>,
    pub len: usize,
}

impl Row {
    /// Creates a new row.
    ///
    /// * `len` - Length of the row.
    pub fn new(len: usize) -> Self {
        Row {
            cells: Row::create_empty_cells(len).into_boxed_slice(),
            len,
        }
    }

    fn create_empty_cells(len: usize) -> Vec<Cell> {
        let mut cells = vec![];

        for _ in 0..len {
            cells.push(Cell {
                text: String::from(" "),
                hl_id: 0,
                double_width: false,
            })
        }

        cells
    }

    /// Returns a cell at a position.
    #[inline]
    pub fn cell_at(&self, at: usize) -> Option<&Cell> {
        self.cells.get(at)
    }

    #[allow(unused)] // Not used currently, but tested.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Clears (resets) the row.
    pub fn clear(&mut self) {
        self.cells = Row::create_empty_cells(self.len).into_boxed_slice();
    }

    pub fn resize(&mut self, new_size: usize) {
        let mut n = self.cells.clone().into_vec();
        n.resize_with(new_size, || Cell {
            text: String::from(" "),
            hl_id: 0,
            double_width: false,
        });

        self.cells = n.into_boxed_slice();
        self.len = self.cells.len();
    }

    /// Clears range from `from` to `to`.
    pub fn clear_range(&mut self, from: usize, to: usize) {
        for i in from..to {
            self.cells[i] = Cell {
                text: String::from(" "),
                hl_id: 0,
                double_width: false,
            }
        }
    }

    /// Copies range from `from` to `to`.
    pub fn copy_range(&self, from: usize, to: usize) -> Vec<Cell> {
        self.cells[from..to].to_vec()
    }

    /// Inserts cells to `at`.
    pub fn insert_at(&mut self, at: usize, cells: Vec<Cell>) {
        for (i, cell) in cells.into_iter().enumerate() {
            self.cells[at + i] = cell;
        }

        assert_eq!(self.cells.len(), self.len);
    }

    /// Updates row. `line` should be coming straight from nvim's 'grid_line'.
    /// event.
    pub fn update(&mut self, line: GridLineSegment) -> Vec<Segment> {
        let col_start = line.col_start as usize;

        // Check where the segment at col_start starts and use that when checking
        // for affected segments. This is so that if col_start is in middle of a
        // ligature, we'll render the whole segment where the ligature might have
        // gotten broken up.
        let range_start =
            if let Some(seg) = self.as_segments(col_start, col_start).first() {
                seg.start
            } else {
                0
            };

        let mut offset = col_start;
        for cell in line.cells.iter() {
            for r in 0..cell.repeat as usize {
                self.cells[offset + r] = Cell {
                    // TODO(ville): Avoid clone here?
                    text: cell.text.clone(),
                    hl_id: cell.hl_id,
                    double_width: cell.double_width,
                };
            }

            offset += cell.repeat as usize;
        }

        assert_eq!(self.cells.len(), self.len);

        self.as_segments(range_start, offset)
    }

    pub fn as_segments(&self, cell_start: usize, end: usize) -> Vec<Segment> {
        let base_hl = self.cells[cell_start].hl_id;
        let base = if let Some((i, _)) = self
            .cells
            .iter()
            .take(cell_start)
            .enumerate()
            .rev()
            .find(|(_, c)| c.hl_id != base_hl)
        {
            // Plus one because we're already "past" from our
            // segment's start.
            i + 1
        } else {
            0
        };

        let mut segs: Vec<Segment> = vec![];
        let mut start = base;

        for (i, cell) in self.cells.iter().enumerate().skip(start) {
            // TODO(ville): Make sure we're not at the middle of a "section".
            if i > end {
                break;
            }

            if let Some(ref mut seg) = segs.last_mut() {
                // If the previous cell is "empty", it means that the cell that
                // came earlier is a double width cell. In such case, we dont
                // want to append _this_ cell into the previous segment, so
                // we avoid rendering double width cells in combination of other
                // cells and thus losing the actual "double width" of a double
                // width cell.
                let prev_is_empty = if let Some(prev) = self.cells.get(i - 1) {
                    prev.text.len() == 0
                } else {
                    false
                };

                if seg.hl_id == cell.hl_id && !prev_is_empty {
                    seg.text.push_str(&cell.text);
                    seg.len += 1;

                    start += 1;
                    continue;
                }
            }

            segs.push(Segment {
                text: cell.text.clone(),
                hl_id: cell.hl_id,
                start,
                len: 1,
            });

            start += 1;
        }

        segs
    }
}

#[cfg(all(feature = "unstable", test))]
mod benches {
    extern crate test;
    use self::test::Bencher;

    use super::*;

    #[bench]
    fn bench_row_update(b: &mut Bencher) {
        let mut row = Row::new(10);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "0".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "1".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "4".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "5".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "6".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "7".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "8".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "9".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
            ],
        );

        b.iter(move || {
            row.clone().update(GridLineSegment {
                grid: 0,
                row: 0,
                col_start: 3,
                cells: vec![
                    nvim_bridge::Cell {
                        text: String::from("1"),
                        hl_id: 1,
                        repeat: 3,
                        double_width: false,
                    },
                    nvim_bridge::Cell {
                        text: String::from("1"),
                        hl_id: 1,
                        repeat: 3,
                        double_width: false,
                    },
                ],
            });
        });
    }

    #[bench]
    fn bench_row_update2(b: &mut Bencher) {
        let mut row = Row::new(300);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "0".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "1".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "4".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "5".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "6".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "7".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "8".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "9".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
            ],
        );

        b.iter(move || {
            row.update(GridLineSegment {
                grid: 0,
                row: 0,
                col_start: 3,
                cells: vec![
                    nvim_bridge::Cell {
                        text: String::from("1"),
                        hl_id: 1,
                        repeat: 3,
                        double_width: false,
                    },
                    nvim_bridge::Cell {
                        text: String::from("1"),
                        hl_id: 2,
                        repeat: 3,
                        double_width: false,
                    },
                ],
            });
        });
    }

    #[bench]
    fn bench_row_clear_range(b: &mut Bencher) {
        let mut row = Row::new(10);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "0".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "1".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "4".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "5".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "6".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "7".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "8".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "9".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
            ],
        );

        b.iter(move || row.clone().clear_range(3, 6));
    }

    #[bench]
    fn bench_insert_at(b: &mut Bencher) {
        b.iter(move || {
            let mut row = Row::new(30);
            row.insert_at(
                5,
                vec![
                    Cell {
                        text: "f".to_string(),
                        hl_id: 0,
                        double_width: false,
                    },
                    Cell {
                        text: "i".to_string(),
                        hl_id: 0,
                        double_width: false,
                    },
                    Cell {
                        text: "r".to_string(),
                        hl_id: 0,
                        double_width: false,
                    },
                    Cell {
                        text: "s".to_string(),
                        hl_id: 0,
                        double_width: false,
                    },
                    Cell {
                        text: "t".to_string(),
                        hl_id: 0,
                        double_width: false,
                    },
                ],
            );
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_row_update() {
        let mut row = Row::new(10);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "0".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "1".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "4".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "5".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "6".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "7".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "8".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "9".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
            ],
        );

        row.update(GridLineSegment {
            grid: 0,
            row: 0,
            col_start: 3,
            cells: vec![
                nvim_bridge::Cell {
                    text: String::from("1"),
                    hl_id: 1,
                    repeat: 3,
                    double_width: false,
                },
                nvim_bridge::Cell {
                    text: String::from("2"),
                    hl_id: 1,
                    repeat: 3,
                    double_width: false,
                },
            ],
        });

        assert_eq!(
            row.cells.iter().map(|c| c.text.clone()).collect::<String>(),
            "0121112229"
        )
    }

    #[test]
    fn test_row_update2() {
        let mut row = Row::new(5);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: " ".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: " ".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "=".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
                Cell {
                    text: "=".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
                Cell {
                    text: "=".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
            ],
        );

        let segments = row.update(GridLineSegment {
            grid: 0,
            row: 0,
            col_start: 4,
            cells: vec![nvim_bridge::Cell {
                text: String::from(" "),
                hl_id: 2,
                repeat: 1,
                double_width: false,
            }],
        });

        assert_eq!(
            row.cells.iter().map(|c| c.text.clone()).collect::<String>(),
            "  == "
        );

        assert_eq!(
            segments,
            vec![
                Segment {
                    text: "==".to_string(),
                    hl_id: 1,
                    start: 2,
                    len: 2,
                },
                Segment {
                    text: " ".to_string(),
                    hl_id: 2,
                    start: 4,
                    len: 1,
                }
            ],
        );
    }

    #[test]
    fn test_row_copy_range() {
        let mut row = Row::new(30);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "f".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "i".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "r".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "s".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "t".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "s".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "e".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "c".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "o".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "n".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "d".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
            ],
        );

        let range = row.copy_range(2, 10);
        assert_eq!(
            range.iter().map(|c| c.text.clone()).collect::<String>(),
            "rstsecon"
        )
    }

    #[test]
    fn test_row_insert_at() {
        let mut row = Row::new(30);
        row.insert_at(
            5,
            vec![
                Cell {
                    text: "f".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "i".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "r".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "s".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "t".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "s".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "e".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "c".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "o".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "n".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "d".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "t".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "h".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "i".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "r".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "d".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
            ],
        );

        assert_eq!(
            row.cells.iter().map(|c| c.text.clone()).collect::<String>(),
            "     firstsecondthird         "
        );
    }

    #[test]
    fn test_row_clear_range() {
        let mut row = Row::new(10);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "0".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "1".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "4".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "5".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "6".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "7".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "8".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
                Cell {
                    text: "9".to_string(),
                    hl_id: 0,
                    double_width: false,
                },
            ],
        );

        row.clear_range(2, 5);

        assert_eq!(
            row.cells.iter().map(|c| c.text.clone()).collect::<String>(),
            "01   56789"
        );
    }

    #[test]
    fn test_row_as_segments() {
        let mut row = Row::new(4);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "1".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
                Cell {
                    text: "1".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 2,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 3,
                    double_width: false,
                },
            ],
        );

        let segments = row.as_segments(0, row.len);

        let first = &segments[0];
        assert_eq!(first.text, "11");
        assert_eq!(first.start, 0);
        assert_eq!(first.len, 2);

        let second = &segments[1];
        assert_eq!(second.text, "2");
        assert_eq!(second.start, 2);
        assert_eq!(second.len, 1);

        let third = &segments[2];
        assert_eq!(third.text, "3");
        assert_eq!(third.start, 3);
        assert_eq!(third.len, 1);
    }

    #[test]
    fn test_row_as_segments_with_cell_start() {
        let mut row = Row::new(10);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "1".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
                Cell {
                    text: "1".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 2,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 2,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 2,
                    double_width: false,
                },
                Cell {
                    text: " ".to_string(),
                    hl_id: 2,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 3,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 3,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 3,
                    double_width: false,
                },
                Cell {
                    text: "3".to_string(),
                    hl_id: 3,
                    double_width: false,
                },
            ],
        );

        let segments = row.as_segments(5, row.len);

        let first = &segments[0];
        assert_eq!(first.text, "222 ");
        assert_eq!(first.start, 2);
        assert_eq!(first.len, 4);

        let second = &segments[1];
        assert_eq!(second.text, "3333");
        assert_eq!(second.start, 6);
        assert_eq!(second.len, 4);
    }

    #[test]
    fn test_row_as_segments_double_width_cell() {
        let mut row = Row::new(4);
        row.insert_at(
            0,
            vec![
                Cell {
                    text: "1".to_string(),
                    hl_id: 1,
                    double_width: true,
                },
                Cell {
                    text: "".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
                Cell {
                    text: "2".to_string(),
                    hl_id: 1,
                    double_width: false,
                },
            ],
        );

        let segments = row.as_segments(0, row.len);

        let first = &segments[0];
        assert_eq!(first.text, "1");
        assert_eq!(first.start, 0);
        assert_eq!(first.len, 2);

        let second = &segments[1];
        assert_eq!(second.text, "2");
        assert_eq!(second.start, 2);
        assert_eq!(second.len, 1);
    }

    #[test]
    fn test_row_grow() {
        let mut row = Row::new(10);
        row.resize(15);

        assert_eq!(row.len, 15);
        assert_eq!(
            row.cells.iter().map(|c| c.text.clone()).collect::<String>(),
            String::from(" ").repeat(15)
        );
    }

    #[test]
    fn test_row_truncate() {
        let mut row = Row::new(10);
        row.resize(5);

        assert_eq!(row.len, 5);
        assert_eq!(
            row.cells.iter().map(|c| c.text.clone()).collect::<String>(),
            String::from(" ").repeat(5)
        );
    }
}
