use nvim_bridge::{GridLineSegment, Cell as NvimCell};

/// Wrapper for a leaf, that tells the leaf's position.
pub struct Segment<'a> {
    pub leaf: &'a mut Leaf,
    pub start: usize,
    pub len: usize,
}

#[derive(Clone)]
pub struct Leaf {
    /// Texts of this leaf.
    text: String,
    /// Hl id of this leaf.
    hl_id: u64,
    /// Length of this leaf in utf8 characters.
    len: usize,
}

/// Leaf of `Rope` (tree) structure.
impl Leaf {
    fn new(text: String, hl_id: u64) -> Self {
        Leaf {
            len: text.chars().count(),
            text,
            hl_id,
        }
    }

    /// Length of this leaf (in utf8 characters).
    #[allow(unused)] // Not used currently, but tested.
    pub fn len(&self) -> usize {
        self.len
    }

    /// This leaf's hl_id.
    pub fn hl_id(&self) -> u64 {
        self.hl_id
    }

    /// Texts of this leaf.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Appends `text` to this leaf.
    pub fn append(&mut self, text: &str) {
        self.len += text.chars().count();
        self.text.push_str(text);
    }

    /// Splits this leaf into two (ropes).
    #[inline]
    fn split(self, at: usize) -> (Rope, Rope) {
        let mut left = String::with_capacity(at);
        let mut right = String::with_capacity(self.len - at);
        for (i, c) in self.text.chars().enumerate() {
            if i < at {
                left.push(c);
            } else {
                right.push(c);
            }
        }

        (
            Rope::Leaf(Leaf::new(left, self.hl_id)),
            Rope::Leaf(Leaf::new(right, self.hl_id)),
        )
    }
}

/// Rope is a "rope"/tree structure that combines leafs by hl_id to make them
/// easier to render as whole "segments". Each leaf/segment can be basically
/// rendered with same hl.
#[derive(Clone)]
pub enum Rope {
    Leaf(Leaf),
    Node(Box<Rope>, Box<Rope>),
}

impl Rope {
    fn new(base: String, hl_id: u64) -> Self {
        Rope::Leaf(Leaf::new(base, hl_id))
    }

    /// Constructs a rope from NvimCells (that are supposedly coming from nvim's
    /// "grid_line" event).
    fn from_nvim_cells(cells: &Vec<NvimCell>) -> Self {
        let mut rope = Rope::new(String::new(), 0);
        for cell in cells {
            let leaf = Leaf::new(cell.text.repeat(cell.repeat as usize), cell.hl_id);
            rope = rope.concat(Rope::Leaf(leaf));
        }

        rope
    }

    /// Returns the current length of the rope.
    #[inline]
    fn len(&self) -> usize {
        match self {
            Rope::Leaf(leaf) => leaf.len,
            Rope::Node(left, right) => {
                left.len() + right.len()
            }
        }
    }

    /// Returns the current weight of the rope. Is the same as Rope.len() in
    /// this implementation.
    #[inline]
    pub fn weight(&self) -> usize {
        match self {
            Rope::Leaf(leaf) => leaf.len,
            Rope::Node(left, right) => {
                right.weight() + left.weight()
            }
        }
    }

    /// Returns the current text of this rope.
    pub fn text(&self) -> String {
        match self {
            Rope::Leaf(leaf) => leaf.text.clone(),
            Rope::Node(left, right) => {
                left.text() + &right.text()
            }
        }
    }

    /// Concatenates another rope to this rope. If we are a leaf, and `other` is
    /// also a leaf, both (`self` and `other`) are combined.
    pub fn concat(self, other: Rope) -> Rope {
        match self {
            Rope::Leaf(mut leaf) => {
                // If the other is just a leaf, check if it's hl_id is the same
                // as ours - if it is, we can just append the text of the other
                // to us.
                match other {
                    Rope::Leaf(other) => {
                        if other.hl_id() == leaf.hl_id() {
                            leaf.append(&other.text);
                            //leaf.text.push_str(&other.text);
                            //leaf.text += &other.text;
                            //leaf.len += other.len;
                            Rope::Leaf(leaf)
                        } else {
                            Rope::Node(
                                Box::new(Rope::Leaf(leaf)),
                                Box::new(Rope::Leaf(other)))
                        }
                    }
                    _ => {
                        Rope::Node(Box::new(Rope::Leaf(leaf)), Box::new(other))
                    }
                }
            },
            Rope::Node(left, right) => {
                left.concat(right.concat(other))
            }
        }
    }

    /// Split us into two, at `at`.
    pub fn split(self, mut at: usize) -> (Rope, Rope) {
        match self {
            Rope::Leaf(leaf) => {
                if at == leaf.len {
                    let hl_id = leaf.hl_id;
                    (Rope::Leaf(leaf), Rope::new(String::new(), hl_id))
                } else {
                    leaf.split(at)
                }
            }
            Rope::Node(left, right) => {
                let weight = left.weight();
                if at == weight {
                    (*left, *right)
                } else if at < weight {
                    let (l, r) = left.split(at);
                    (l, r.concat(*right))
                } else {
                    at = at - weight;
                    let (l, r) = right.split(at);
                    (left.concat(l), r)
                }
            }
        }
    }

    /// Returns our leafs as mutable references.
    pub fn leafs_mut(&mut self) -> Vec<&mut Leaf> {
        match self {
            Rope::Leaf(leaf) => {
                vec!(leaf)
            }
            Rope::Node(left, right) => {
                let mut left = left.leafs_mut();
                left.append(&mut right.leafs_mut());
                left
            }
        }
    }

    /// Returns our leafs as reference.
    pub fn leafs(&self) -> Vec<&Leaf> {
        match self {
            Rope::Leaf(leaf) => {
                vec!(leaf)
            }
            Rope::Node(left, right) => {
                let mut left = left.leafs();
                left.append(&mut right.leafs());
                left
            }
        }
    }

    /// Returns leaf at `at`.
    pub fn leaf_at(&self, at: usize) -> &Leaf {
        match self {
            Rope::Leaf(leaf) => {
                &leaf
            }
            Rope::Node(left, right) => {
                let weight = left.weight();
                if at <= weight {
                    left.leaf_at(at)
                } else {
                    let at = at - weight;
                    right.leaf_at(at)
                }
            }
        }
    }

    /// Combines leafs together, based on hl_id.
    pub fn combine_leafs(&self) -> Rope {
        assert!(self.len() > 0,
            "Rope needs to have length greater than 0 in order to be combine_leafs");

        let mut rope = None;

        let leafs = self.leafs();
        for leaf in leafs {
            if leaf.len == 0 {
                continue;
            }

            if rope.is_none() {
                rope = Some(Rope::Leaf(leaf.clone()));
            } else {
                rope = Some(rope.unwrap().concat(Rope::Leaf(leaf.clone())));
            }
        }

        rope.unwrap()
    }
}

/// Row, as in one row in a grid. Internally has a rope/tree structure.
#[derive(Clone)]
pub struct Row {
    /// Actual contents of a row. This is an option, so we can temporally have
    /// it as an none when restructuring it. This should only _not_ be none when
    /// entering or leaving a function.
    rope: Option<Rope>,
    /// Length of the row.
    len: usize,
}

impl Row {
    /// Creates a new row.
    ///
    /// * `len` - Length of the row.
    pub fn new(len: usize) -> Self {
        Row {
            rope: Some(Rope::new(" ".repeat(len), 0)),
            len: len,
        }
    }

    /// Returns the whole text of the row.
    #[allow(unused)] // Not used currently, but tested.
    pub fn text(&self) -> String {
        self.rope.as_ref().unwrap().text()
    }

    /// Returns a leaf at a position.
    #[inline]
    pub fn leaf_at(&self, at: usize) -> &Leaf {
        self.rope.as_ref().unwrap().leaf_at(at)
    }

    #[allow(unused)] // Not used currently, but tested.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Clears (resets) the row.
    pub fn clear(&mut self) {
        self.rope = Some(Rope::new(" ".repeat(self.len), 0));
    }

    /// Clears range from `from` to `to`.
    pub fn clear_range(&mut self, from: usize, to: usize) {
        let middle_len = to - from;

        let (left, right) = self.rope.take().unwrap().split(from);
        let (_, right) = right.split(middle_len);
        let middle = Rope::new(" ".repeat(middle_len), 0);
        let left = left.concat(middle);

        self.rope = Some(left.concat(right));
    }

    /// Copies range from `from` to `to`.
    pub fn copy_range(&self, from: usize, to: usize) -> Rope {
        let (_, rope) = self.rope.as_ref().unwrap().clone().split(from);
        let (rope, _) = rope.split(to - from);
        rope
    }

    /// Inserts rope to `at`. What ever is between `at` and `rope.len()` is
    /// replaced.
    pub fn insert_rope_at(&mut self, at: usize, rope: Rope) {

        let (left, right) = self.rope.take().unwrap().split(at);
        let (_, right) = right.split(rope.len());
        self.rope = Some(left.concat(rope).concat(right));

        assert_eq!(self.rope.as_ref().unwrap().len(), self.len);
    }

    /// Updates row. `line` should be coming straight from nvim's 'grid_line'.
    /// event.
    pub fn update(&mut self, line: &GridLineSegment) -> Vec<Segment> {

        // Construct a rope from give cells in `line` and insert it into us.
        let other = Rope::from_nvim_cells(&line.cells);
        let other_len = other.len();
        let col_start = line.col_start as usize;
        let other_end = col_start + other_len;
        self.insert_rope_at(col_start, other);

        // Combine leafs so they are "intact".
        self.rope = Some(self.rope.take().unwrap().combine_leafs());
        assert_eq!(self.rope.as_ref().unwrap().len(), self.len);

        // Compute segments that were affected by this update and return
        // them to the caller.
        let mut segs = vec!();
        let mut start = 0;
        let rope = self.rope.as_mut().unwrap();
        let leafs = rope.leafs_mut();
        for leaf in leafs {
            // If we're past the affected range, break early.
            if start > other_end {
                break;
            }

            let len = leaf.len;
            let end = start + len;

            // If we're not yet in the affected range, continue to the next leaf.
            if end < col_start {
                start = end;
                continue;
            }

            segs.push(Segment {
                leaf: leaf,
                start: start,
                len: len,
            });

            start = end;
        }

        segs
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use self::test::Bencher;

    use super::*;

    #[bench]
    fn bench_row_update(b: &mut Bencher) {
        let mut row = Row::new(10);
        row.insert_rope_at(0, Rope::new(String::from("1234567890"), 0));

        b.iter(move || {
            row.clone()
                .update(&GridLineSegment{
                    grid: 0,
                    row: 0,
                    col_start: 3,
                    cells: vec!(
                        nvim_bridge::Cell {
                            text: String::from("1"),
                            hl_id: 1,
                            repeat: 3,
                        },
                        nvim_bridge::Cell {
                            text: String::from("1"),
                            hl_id: 1,
                            repeat: 3,
                        },
                    )});
        });
    }

    #[bench]
    fn bench_row_update2(b: &mut Bencher) {
        let mut row = Row::new(10);
        row.insert_rope_at(0, Rope::new(String::from("1234567890"), 0));

        b.iter(move || {
            row.clone()
                .update(&GridLineSegment{
                    grid: 0,
                    row: 0,
                    col_start: 3,
                    cells: vec!(
                        nvim_bridge::Cell {
                            text: String::from("1"),
                            hl_id: 1,
                            repeat: 3,
                        },
                        nvim_bridge::Cell {
                            text: String::from("1"),
                            hl_id: 2,
                            repeat: 3,
                        },
                    )});
        });
    }

    #[test]
    fn test_rope_from_nvim_cells() {
        let cells = vec!(
            nvim_bridge::Cell {
                text: String::from("1"),
                hl_id: 1,
                repeat: 3,
            },
            nvim_bridge::Cell {
                text: String::from("2"),
                hl_id: 2,
                repeat: 3,
            });

        let rope = Rope::from_nvim_cells(&cells);

        assert_eq!(rope.text(), "111222");
        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 3);
        assert_eq!(leafs[1].hl_id, 1);
        assert_eq!(leafs[2].hl_id, 2);

        let rope = rope.combine_leafs();
        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 2);
        assert_eq!(leafs[0].hl_id, 1);
        assert_eq!(leafs[1].hl_id, 2);
    }

    #[bench]
    fn bench_row_clear_range(b: &mut Bencher) {
        let mut row = Row::new(10);
        row.insert_rope_at(0, Rope::new(String::from("1234567890"), 0));

        b.iter(move || {
            row.clone().clear_range(3, 6)
        });
    }

    #[bench]
    fn bench_rope_concat(b: &mut Bencher) {

        b.iter(move || {
            let rope = Rope::new(String::from("first"), 0);
            let rope2 = Rope::new(String::from("second"), 0);
            rope.concat(rope2)
        });
    }

    #[bench]
    fn bench_rope_split(b: &mut Bencher) {
        let rope = Rope::new(String::from("first"), 0);
        let rope = rope.concat(Rope::new(String::from("second"), 0));
        let rope = rope.concat(Rope::new(String::from("third"), 2));
        let rope = rope.concat(Rope::new(String::from("fourth"), 3));

        //let rope = rope.combine_leafs();

        b.iter(move || {
            rope.clone().split(3)
        });
    }

    #[bench]
    fn bench_insert_rope(b: &mut Bencher) {

        b.iter(move || {
            let mut row = Row::new(30);
            let rope = Rope::new(String::from("first"), 0);
            row.insert_rope_at(5, rope)
        });
    }

    #[bench]
    fn bench_leaf_split(b: &mut Bencher) {

        b.iter(move || {
            let mut leaf = Leaf::new(String::from("123123123"), 0);
            leaf.split(4)
        });
    }

    #[test]
    fn test_leaf_split() {
        let mut leaf = Leaf::new(String::from("1234"), 0);
        let (left, right) = leaf.split(2);
        assert_eq!("12", left.text());
        assert_eq!("34", right.text());
    }

    #[test]
    fn test_leaf_len() {
        let leaf = Leaf::new(String::from("123"), 0);
        assert_eq!(leaf.len, 3);
        let leaf = Leaf::new(String::from("✗ä"), 0);
        assert_eq!(leaf.len, 2);
    }

    #[test]
    fn test_rope_len() {
        let left = Rope::Leaf(Leaf::new(String::from("123"), 0));
        let right = Rope::Leaf(Leaf::new(String::from("✗ä"), 0));
        let rope = Rope::Node(
            Box::new(left),
            Box::new(right));

        assert_eq!(rope.len(), 5);
    }

    #[test]
    fn test_rope_weight() {
        let left = Rope::Leaf(Leaf::new(String::from("123"), 0));
        let right = Rope::Leaf(Leaf::new(String::from("✗ä"), 0));
        let rope = Rope::Node(
            Box::new(left),
            Box::new(right));

        assert_eq!(rope.weight(), 5);
    }

    #[test]
    fn test_rope_concat() {
        let rope = Rope::new(String::from("Hello,"), 0);
        let other = Rope::new(String::from(" World!"), 0);
        let rope = rope.concat(other);
        assert_eq!(rope.text(), "Hello, World!");
        let other = Rope::new(String::from(" other"), 0);
        let rope = rope.concat(other);
        assert_eq!(rope.text(), "Hello, World! other");
    }

    #[test]
    fn test_rope_split() {
        let rope = Rope::new(String::from("123✗ä"), 0);

        let (left, right) = rope.clone().split(3);
        assert_eq!(left.len(), 3);
        assert_eq!(right.len(), 2);

        let (left, right) = rope.clone().split(4);
        assert_eq!(left.len(), 4);
        assert_eq!(right.len(), 1);

        let (left, right) = rope.clone().split(1);
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 4);

        let rope = Rope::new(String::from("1234✗ä"), 0);

        let (left, right) = rope.clone().split(1);
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 5);

        let (left, right) = rope.clone().split(0);
        assert_eq!(left.len(), 0);
        assert_eq!(right.len(), 6);

        let (left, right) = rope.clone().split(6);
        assert_eq!(left.len(), 6);
        assert_eq!(right.len(), 0);
    }

    #[test]
    fn test_rope_leafs() {
        let rope = Rope::new(String::from("first"), 1);
        let rope = rope.concat(Rope::new(String::from("second"), 2));
        let rope = rope.concat(Rope::new(String::from("third"), 3));

        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 3);
        assert_eq!(leafs.get(0).unwrap().text, "first");
        assert_eq!(leafs.get(1).unwrap().text, "second");
        assert_eq!(leafs.get(2).unwrap().text, "third");

        let rope = Rope::new(String::from("first"), 0);
        let rope = rope.concat(Rope::new(String::from("second"), 0));

        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 1);
        assert_eq!(leafs.get(0).unwrap().text, "firstsecond");
    }

    #[test]
    fn test_row_copy_range() {
        let mut row = Row::new(30);
        let rope = Rope::new(String::from("first"), 0);
        let rope = rope.concat(Rope::new(String::from("second"), 0));
        let rope = rope.concat(Rope::new(String::from("third"), 0));
        row.rope = Some(rope);

        let range = row.copy_range(2, 10);
        assert_eq!(range.text(), "rstsecon")
    }

    #[test]
    fn test_row_insert_rope_at() {
        let mut row = Row::new(30);
        let rope = Rope::new(String::from("first"), 0);
        let rope = rope.concat(Rope::new(String::from("second"), 0));
        let rope = rope.concat(Rope::new(String::from("third"), 0));

        row.insert_rope_at(5, rope);

        assert_eq!(row.rope.unwrap().text(), "     firstsecondthird         ");
    }

    #[test]
    fn test_row_clear_range() {
        let mut row = Row::new(10);
        row.rope = Some(Rope::new(String::from("0123456789"), 0));

        row.clear_range(2, 5);

        assert_eq!(row.rope.unwrap().text(), "01   56789");
    }

    #[test]
    fn test_rope_leaf_at() {
        let rope = Rope::new(String::from("first"), 1);
        let rope = rope.concat(Rope::new(String::from("second"), 2));
        let rope = rope.concat(Rope::new(String::from("third"), 3));

        assert_eq!(rope.leaf_at(2).hl_id, 1);
        assert_eq!(rope.leaf_at(0).hl_id, 1);
        assert_eq!(rope.leaf_at(5).hl_id, 1);
        assert_eq!(rope.leaf_at(6).hl_id, 2);
        assert_eq!(rope.leaf_at(8).hl_id, 2);
        assert_eq!(rope.leaf_at(11).hl_id, 2);
        assert_eq!(rope.leaf_at(13).hl_id, 3);
    }

    #[test]
    fn test_rope_combine_leafs() {
        let rope = Rope::new(String::from("first"), 0);
        let rope = rope.concat(
            Rope::Leaf(Leaf::new(String::from("second"), 1)));
        let rope = rope.concat(
            Rope::Leaf(Leaf::new(String::from("third"), 0)));
        let rope = rope.concat(
            Rope::Leaf(Leaf::new(String::from("fourth"), 1)));

        assert_eq!(rope.leafs().len(), 4);

        let (left, right) = rope.split(5);
        let rope = left.concat(right.split(6).1);
        assert_eq!(rope.text(), "firstthirdfourth");
        assert_eq!(rope.leafs().len(), 3);

        let rope = rope.combine_leafs();
        assert_eq!(rope.text(), "firstthirdfourth");

        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 2);
        assert_eq!(leafs[0].text, "firstthird");
        assert_eq!(leafs[1].text, "fourth");
    }

    #[test]
    fn test_rope_combine_leafs2() {
        let rope = Rope::new(String::from(""), 3);
        let rope = rope.concat(
            Rope::Leaf(Leaf::new(String::from("first"), 0)));
        let rope = rope.concat(
            Rope::Leaf(Leaf::new(String::from("second"), 1)));
        let rope = rope.concat(
            Rope::Leaf(Leaf::new(String::from("third"), 0)));
        let rope = rope.concat(
            Rope::Leaf(Leaf::new(String::from(""), 1)));

        assert_eq!(rope.leafs().len(), 5);

        let (left, right) = rope.split(5);
        let rope = left.concat(right.split(6).1);
        assert_eq!(rope.text(), "firstthird");
        assert_eq!(rope.leafs().len(), 4);

        let rope = rope.combine_leafs();
        assert_eq!(rope.text(), "firstthird");

        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 1);
        assert_eq!(leafs[0].text, "firstthird");
    }
}
