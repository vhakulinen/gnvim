use gtk::prelude::*;

pub trait IterChildren {
    fn iter_children(&self) -> ChildIter;
}

impl<T> IterChildren for T
where
    T: IsA<gtk::Widget>,
{
    /// Iterates over children. Note that the iterator is not stable, because the
    /// underlying data might change.
    fn iter_children(&self) -> ChildIter {
        ChildIter {
            child: self.first_child(),
        }
    }
}

/// Unstable iterator over widget's children.
pub struct ChildIter {
    child: Option<gtk::Widget>,
}

impl Iterator for ChildIter {
    type Item = gtk::Widget;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.child.take();

        if let Some(ref next) = next {
            self.child = next.next_sibling();
        }

        next
    }
}

#[cfg(test)]
mod tests {
    use super::IterChildren;
    use gtk::prelude::*;

    #[test]
    fn test_child_iter() {
        gtk::init().unwrap();

        let parent = gtk::Label::new(None);

        let child1 = gtk::Label::new(None);
        child1.set_parent(&parent);
        let child2 = gtk::Label::new(None);
        child2.set_parent(&parent);
        let child3 = gtk::Label::new(None);
        child3.set_parent(&parent);

        let mut iter = parent.iter_children();
        assert_eq!(iter.next(), Some(child1.upcast()));
        assert_eq!(iter.next(), Some(child2.upcast()));
        assert_eq!(iter.next(), Some(child3.upcast()));
        assert_eq!(iter.next(), None);

        // Clean up.
        while let Some(child) = parent.first_child() {
            child.unparent();
        }
    }
}
