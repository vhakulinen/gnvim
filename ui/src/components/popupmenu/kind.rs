use std::collections::HashMap;

use glib;

use crate::{
    api,
    colors::{Colors, Highlight, HlAttr, HlGroup},
};

#[derive(Default)]
pub struct Kinds(HashMap<String, Kind>);

impl Kinds {
    pub fn get(&mut self, kind: &str, colors: &Colors) -> &Kind {
        if !self.0.contains_key(kind) {
            self.0.insert(
                kind.to_owned(),
                Kind::new(
                    &colors.get_hl_group(&HlGroup::Pmenu),
                    &colors.get_hl_group(&HlGroup::PmenuSel),
                    kind,
                ),
            );
        }

        self.0.get(kind).unwrap()
    }

    pub fn from_api(apikinds: HashMap<String, api::PopupmenuKind>, colors: &Colors) -> Self {
        let kinds = apikinds
            .iter()
            .map(|(k, v)| {
                let hl: Option<HlAttr> = v.hl.as_ref().map(From::from);
                let hl_sel: Option<HlAttr> = v.sel_hl.as_ref().map(From::from);
                let kind = Kind::new(
                    &Highlight::new(
                        hl.as_ref()
                            .or_else(|| colors.get_hl_grpup_attr(&HlGroup::Pmenu)),
                        &colors,
                    ),
                    &Highlight::new(
                        hl_sel
                            .as_ref()
                            .or_else(|| colors.get_hl_grpup_attr(&HlGroup::PmenuSel)),
                        &colors,
                    ),
                    v.label.as_ref().unwrap_or(k),
                );

                (k.to_owned(), kind)
            })
            .collect::<HashMap<String, Kind>>();

        Self(kinds)
    }
}

glib::wrapper! {
    pub struct Kind(ObjectSubclass<imp::Kind>);
}

impl Kind {
    fn new(hl: &Highlight, hl_sel: &Highlight, label: &str) -> Self {
        glib::Object::builder()
            .property("normal", hl.pango_markup(label))
            .property("selected", hl_sel.pango_markup(label))
            .build()
    }
}

impl Default for Kind {
    fn default() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::{prelude::*, subclass::prelude::*};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Kind)]
    pub struct Kind {
        #[property(get, set)]
        pub normal: RefCell<String>,
        #[property(get, set)]
        pub selected: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Kind {
        const NAME: &'static str = "PopupmenuKind";
        type Type = super::Kind;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Kind {}
}
