use nvim_bridge::GridLineSegment;

pub struct Segment<'a> {
    pub leaf: &'a Leaf,
    pub start: usize,
    pub len: usize,
}

#[derive(Clone)]
pub struct Leaf {
    text: String,
    hl_id: u64,
}

impl Leaf {
    fn new(text: String, hl_id: u64) -> Self {
        Leaf {
            text,
            hl_id,
        }
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }

    fn weight(&self) -> usize {
        self.len()
    }

    #[inline]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    #[inline]
    pub fn hl_id(&self) -> u64 {
        self.hl_id
    }

    fn split(self, at: usize) -> (Rope, Rope) {
        //assert!(at < self.len(), format!("Leaf split bounds ({} < {})", at, self.len()));
        let left = self.text.chars().take(at).collect::<String>();
        let right = self.text.chars().skip(at).collect::<String>();

        (
            Rope::new(left, self.hl_id),
            Rope::new(right, self.hl_id),
        )
    }
}

#[derive(Clone)]
pub enum Rope {
    Leaf(Leaf),
    Node(Box<Rope>, Box<Rope>),
}

impl Rope {
    fn new(base: String, hl_id: u64) -> Self {
        Rope::Leaf(Leaf::new(base, hl_id))
    }

    fn len(&self) -> usize {
        match self {
            Rope::Leaf(leaf) => leaf.len(),
            Rope::Node(left, right) => {
                left.len() + right.len()
            }
        }
    }

    pub fn weight(&self) -> usize {
        match self {
            Rope::Leaf(leaf) => leaf.weight(),
            Rope::Node(left, right) => {
                right.weight() + left.weight()
            }
        }
    }

    pub fn text(&self) -> String {
        match self {
            Rope::Leaf(leaf) => leaf.text(),
            Rope::Node(left, right) => {
                left.text() + &right.text()
            }
        }
    }

    pub fn concat(self, other: Rope) -> Rope {
        match self {
            Rope::Leaf(mut leaf) => {
                // If the other is just a leaf, check if it's hl_id is the same
                // as ours - if it is, we can just append the text of the other
                // to us.
                match other {
                    Rope::Leaf(other) => {
                        if other.hl_id == leaf.hl_id {
                            leaf.text += &other.text;
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
                let right = right.concat(other);
                left.concat(right)
            }
        }
    }

    pub fn split(self, at: usize) -> (Rope, Rope) {
        //assert!(at <= self.len(), format!("Bounds check on rope split ({} < {})", at, self.len()));
        match self {
            Rope::Leaf(leaf) => {
                if at == leaf.len() {
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
                    let at = at - left.len();
                    let (l, r) = right.split(at);
                    (left.concat(l), r)
                }
            }
        }
    }

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
                    let at = at - left.len();
                    right.leaf_at(at)
                }
            }
        }
    }

    pub fn optimize(&self) -> Rope {
        assert!(self.len() > 0,
            "Rope needs to have lenght greater than 0 inorder to be optimized");

        let mut rope = None;

        let mut leafs = self.leafs();
        for leaf in leafs {
            if leaf.len() == 0 {
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

#[derive(Clone)]
pub struct Row {
    rope: Rope,
    len: usize,
}

impl Row {
    pub fn new(len: usize) -> Self {
        Row {
            rope: Rope::new(" ".repeat(len), 0),
            len: len,
        }
    }

    //pub fn debug_print(&self) {
        //self.rope.debug_print();
    //}

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.rope = Rope::new(" ".repeat(self.len), 0);
    }

    pub fn clear_range(&mut self, from: usize, to: usize) {
        let (left, right) = self.rope.clone().split(from);
        let (_, right) = right.split(to - from);
        let middle = Rope::new(" ".repeat(to - from), 0);
        let left = left.concat(middle);
        self.rope = left.concat(right);
    }

    pub fn copy_range(&self, from: usize, to: usize) -> Rope {
        let (_, rope) = self.rope.clone().split(from);
        let (rope, _) = rope.split(to - from);
        rope
    }

    pub fn insert_rope_at(&mut self, at: usize, rope: Rope) {

        let (left, right) = self.rope.clone().split(at);
        let (_, right) = right.split(rope.len());
        self.rope = left.concat(rope).concat(right);

        assert_eq!(self.rope.len(), self.len);
    }


    pub fn update(&mut self, line: &GridLineSegment) -> Vec<Segment> {
        let mut at = line.col_start as usize;
        for cell in &line.cells {
            let text = cell.text.repeat(cell.repeat as usize);
            let len = text.chars().count();

            let other = Rope::new(text, cell.hl_id.unwrap_or(self.rope.leaf_at(at).hl_id));
            self.insert_rope_at(at, other);

            at += len;
        }

        println!("LEAF COUNT PRE: {}", self.rope.leafs().len());
        self.rope = self.rope.optimize();

        assert_eq!(self.rope.len(), self.len);

        let mut segs = vec!();
        let mut start = 0;
        let leafs = self.rope.leafs();
        println!("LEAF COUNT: {}", leafs.len());
        for leaf in leafs {
            let len = leaf.len();
            segs.push(Segment {
                leaf: leaf,
                start: start,
                len: len,
            });
            start += len;
        }

        segs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(leaf.len(), 3);
        let leaf = Leaf::new(String::from("✗ä"), 0);
        assert_eq!(leaf.len(), 2);
    }

    #[test]
    fn test_leaf_weight() {
        let leaf = Leaf::new(String::from("123"), 0);
        assert_eq!(leaf.weight(), 3);
        let leaf = Leaf::new(String::from("✗ä"), 0);
        assert_eq!(leaf.weight(), 2);
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
        assert_eq!(leafs.get(0).unwrap().text(), "first");
        assert_eq!(leafs.get(1).unwrap().text(), "second");
        assert_eq!(leafs.get(2).unwrap().text(), "third");

        let rope = Rope::new(String::from("first"), 0);
        let rope = rope.concat(Rope::new(String::from("second"), 0));

        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 1);
        assert_eq!(leafs.get(0).unwrap().text(), "firstsecond");
    }

    #[test]
    fn test_row_copy_range() {
        let mut row = Row::new(30);
        let rope = Rope::new(String::from("first"), 0);
        let rope = rope.concat(Rope::new(String::from("second"), 0));
        let rope = rope.concat(Rope::new(String::from("third"), 0));
        row.rope = rope;

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

        assert_eq!(row.rope.text(), "     firstsecondthird         ");
    }

    #[test]
    fn test_row_clear_range() {
        let mut row = Row::new(10);
        row.rope = Rope::new(String::from("0123456789"), 0);

        row.clear_range(2, 5);

        assert_eq!(row.rope.text(), "01   56789");
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
    fn test_rope_optimize() {
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

        let rope = rope.optimize();
        assert_eq!(rope.text(), "firstthirdfourth");

        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 2);
        assert_eq!(leafs[0].text(), "firstthird");
        assert_eq!(leafs[1].text(), "fourth");
    }

    #[test]
    fn test_rope_optimize2() {
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

        let rope = rope.optimize();
        assert_eq!(rope.text(), "firstthird");

        let leafs = rope.leafs();
        assert_eq!(leafs.len(), 1);
        assert_eq!(leafs[0].text(), "firstthird");
    }
}
