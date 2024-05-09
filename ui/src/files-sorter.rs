use std::cell::RefCell;

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

glib::wrapper! {
    pub struct FilesSorter(ObjectSubclass<Private>)
        @extends gtk::Sorter;
}

impl Default for FilesSorter {
    fn default() -> Self {
        glib::Object::new()
    }
}

#[derive(Default)]
pub struct Private {}

#[glib::object_subclass]
impl ObjectSubclass for Private {
    const NAME: &'static str = "FilesSorter";
    type Type = FilesSorter;
    type ParentType = gtk::Sorter;
}

impl ObjectImpl for Private {}

impl SorterImpl for Private {
    fn compare(&self, a: &glib::Object, b: &glib::Object) -> gtk::Ordering {
        use gio::FileType;

        let a = a
            .downcast_ref::<gio::FileInfo>()
            .expect("Sorted object must be type of gio::FileInfo");
        let b = b
            .downcast_ref::<gio::FileInfo>()
            .expect("Sorted object must be type of gio::FileInfo");

        match (a.file_type(), b.file_type()) {
            (FileType::Directory, FileType::Directory) => a.name().cmp(&b.name()).into(),
            (FileType::Regular, FileType::Regular) => a.name().cmp(&b.name()).into(),
            (FileType::Directory, _) => gtk::Ordering::Smaller,
            (_, FileType::Directory) => gtk::Ordering::Larger,
            (_, _) => a.name().cmp(&b.name()).into(),
        }
    }
}
