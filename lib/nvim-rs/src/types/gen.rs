use super::manual::*;
use std::fmt::Display;
#[derive(Debug)]
pub struct ModeInfoSet {
    pub enabled: bool,
    pub cursor_styles: Vec<ModeInfo>,
}
impl<'de> serde::Deserialize<'de> for ModeInfoSet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ModeInfoSet;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid ModeInfoSet")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = ModeInfoSet {
                    enabled: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    cursor_styles: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct ModeChange {
    pub mode: String,
    pub mode_idx: i64,
}
impl<'de> serde::Deserialize<'de> for ModeChange {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ModeChange;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid ModeChange")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = ModeChange {
                    mode: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    mode_idx: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct SetTitle {
    pub title: String,
}
impl<'de> serde::Deserialize<'de> for SetTitle {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = SetTitle;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid SetTitle")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = SetTitle {
                    title: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct SetIcon {
    pub icon: String,
}
impl<'de> serde::Deserialize<'de> for SetIcon {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = SetIcon;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid SetIcon")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = SetIcon {
                    icon: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct Screenshot {
    pub path: String,
}
impl<'de> serde::Deserialize<'de> for Screenshot {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Screenshot;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid Screenshot")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = Screenshot {
                    path: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct Chdir {
    pub path: String,
}
impl<'de> serde::Deserialize<'de> for Chdir {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Chdir;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid Chdir")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = Chdir {
                    path: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct UpdateFg {
    pub fg: i64,
}
impl<'de> serde::Deserialize<'de> for UpdateFg {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UpdateFg;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid UpdateFg")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = UpdateFg {
                    fg: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct UpdateBg {
    pub bg: i64,
}
impl<'de> serde::Deserialize<'de> for UpdateBg {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UpdateBg;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid UpdateBg")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = UpdateBg {
                    bg: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct UpdateSp {
    pub sp: i64,
}
impl<'de> serde::Deserialize<'de> for UpdateSp {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UpdateSp;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid UpdateSp")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = UpdateSp {
                    sp: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct Resize {
    pub width: i64,
    pub height: i64,
}
impl<'de> serde::Deserialize<'de> for Resize {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Resize;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid Resize")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = Resize {
                    width: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    height: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct CursorGoto {
    pub row: i64,
    pub col: i64,
}
impl<'de> serde::Deserialize<'de> for CursorGoto {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CursorGoto;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid CursorGoto")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = CursorGoto {
                    row: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    col: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct HighlightSet {
    pub attrs: Dictionary,
}
impl<'de> serde::Deserialize<'de> for HighlightSet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = HighlightSet;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid HighlightSet")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = HighlightSet {
                    attrs: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct Put {
    pub str: String,
}
impl<'de> serde::Deserialize<'de> for Put {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Put;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid Put")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = Put {
                    str: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct SetScrollRegion {
    pub top: i64,
    pub bot: i64,
    pub left: i64,
    pub right: i64,
}
impl<'de> serde::Deserialize<'de> for SetScrollRegion {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = SetScrollRegion;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid SetScrollRegion")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = SetScrollRegion {
                    top: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    bot: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    left: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    right: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct Scroll {
    pub count: i64,
}
impl<'de> serde::Deserialize<'de> for Scroll {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Scroll;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid Scroll")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = Scroll {
                    count: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct DefaultColorsSet {
    pub rgb_fg: i64,
    pub rgb_bg: i64,
    pub rgb_sp: i64,
    pub cterm_fg: i64,
    pub cterm_bg: i64,
}
impl<'de> serde::Deserialize<'de> for DefaultColorsSet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = DefaultColorsSet;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid DefaultColorsSet")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = DefaultColorsSet {
                    rgb_fg: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    rgb_bg: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    rgb_sp: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    cterm_fg: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    cterm_bg: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct HlAttrDefine {
    pub id: i64,
    pub rgb_attrs: HlAttr,
    pub cterm_attrs: HlAttr,
    pub info: Vec<rmpv::Value>,
}
impl<'de> serde::Deserialize<'de> for HlAttrDefine {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = HlAttrDefine;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid HlAttrDefine")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = HlAttrDefine {
                    id: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    rgb_attrs: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    cterm_attrs: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    info: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct HlGroupSet {
    pub name: String,
    pub id: i64,
}
impl<'de> serde::Deserialize<'de> for HlGroupSet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = HlGroupSet;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid HlGroupSet")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = HlGroupSet {
                    name: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    id: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct GridResize {
    pub grid: i64,
    pub width: i64,
    pub height: i64,
}
impl<'de> serde::Deserialize<'de> for GridResize {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GridResize;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid GridResize")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = GridResize {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    width: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    height: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct GridClear {
    pub grid: i64,
}
impl<'de> serde::Deserialize<'de> for GridClear {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GridClear;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid GridClear")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = GridClear {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct GridCursorGoto {
    pub grid: i64,
    pub row: i64,
    pub col: i64,
}
impl<'de> serde::Deserialize<'de> for GridCursorGoto {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GridCursorGoto;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid GridCursorGoto")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = GridCursorGoto {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    row: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    col: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct GridLine {
    pub grid: i64,
    pub row: i64,
    pub col_start: i64,
    pub data: Vec<GridLineData>,
    pub wrap: bool,
}
impl<'de> serde::Deserialize<'de> for GridLine {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GridLine;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid GridLine")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = GridLine {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    row: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    col_start: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    data: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    wrap: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct GridScroll {
    pub grid: i64,
    pub top: i64,
    pub bot: i64,
    pub left: i64,
    pub right: i64,
    pub rows: i64,
    pub cols: i64,
}
impl<'de> serde::Deserialize<'de> for GridScroll {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GridScroll;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid GridScroll")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = GridScroll {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    top: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    bot: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    left: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    right: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                    rows: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5usize, &self))?,
                    cols: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(6usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct GridDestroy {
    pub grid: i64,
}
impl<'de> serde::Deserialize<'de> for GridDestroy {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = GridDestroy;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid GridDestroy")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = GridDestroy {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinPos {
    pub grid: i64,
    pub win: Window,
    pub startrow: i64,
    pub startcol: i64,
    pub width: i64,
    pub height: i64,
}
impl<'de> serde::Deserialize<'de> for WinPos {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinPos;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinPos")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinPos {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    win: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    startrow: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    startcol: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    width: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                    height: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinFloatPos {
    pub grid: i64,
    pub win: Window,
    pub anchor: String,
    pub anchor_grid: i64,
    pub anchor_row: f64,
    pub anchor_col: f64,
    pub focusable: bool,
    pub zindex: i64,
}
impl<'de> serde::Deserialize<'de> for WinFloatPos {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinFloatPos;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinFloatPos")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinFloatPos {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    win: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    anchor: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    anchor_grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    anchor_row: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                    anchor_col: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5usize, &self))?,
                    focusable: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(6usize, &self))?,
                    zindex: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(7usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinExternalPos {
    pub grid: i64,
    pub win: Window,
}
impl<'de> serde::Deserialize<'de> for WinExternalPos {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinExternalPos;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinExternalPos")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinExternalPos {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    win: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinHide {
    pub grid: i64,
}
impl<'de> serde::Deserialize<'de> for WinHide {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinHide;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinHide")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinHide {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinClose {
    pub grid: i64,
}
impl<'de> serde::Deserialize<'de> for WinClose {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinClose;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinClose")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinClose {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct MsgSetPos {
    pub grid: i64,
    pub row: i64,
    pub scrolled: bool,
    pub sep_char: String,
}
impl<'de> serde::Deserialize<'de> for MsgSetPos {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MsgSetPos;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid MsgSetPos")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = MsgSetPos {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    row: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    scrolled: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    sep_char: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinViewport {
    pub grid: i64,
    pub win: Window,
    pub topline: i64,
    pub botline: i64,
    pub curline: i64,
    pub curcol: i64,
    pub line_count: i64,
    pub scroll_delta: i64,
}
impl<'de> serde::Deserialize<'de> for WinViewport {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinViewport;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinViewport")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinViewport {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    win: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    topline: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    botline: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    curline: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                    curcol: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5usize, &self))?,
                    line_count: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(6usize, &self))?,
                    scroll_delta: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(7usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinViewportMargins {
    pub grid: i64,
    pub win: Window,
    pub top: i64,
    pub bottom: i64,
    pub left: i64,
    pub right: i64,
}
impl<'de> serde::Deserialize<'de> for WinViewportMargins {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinViewportMargins;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinViewportMargins")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinViewportMargins {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    win: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    top: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    bottom: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    left: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                    right: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WinExtmark {
    pub grid: i64,
    pub win: Window,
    pub ns_id: i64,
    pub mark_id: i64,
    pub row: i64,
    pub col: i64,
}
impl<'de> serde::Deserialize<'de> for WinExtmark {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WinExtmark;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WinExtmark")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WinExtmark {
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    win: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    ns_id: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    mark_id: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    row: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                    col: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct PopupmenuShow {
    pub items: Vec<PopupmenuItem>,
    pub selected: i64,
    pub row: i64,
    pub col: i64,
    pub grid: i64,
}
impl<'de> serde::Deserialize<'de> for PopupmenuShow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PopupmenuShow;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid PopupmenuShow")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = PopupmenuShow {
                    items: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    selected: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    row: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    col: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    grid: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct PopupmenuSelect {
    pub selected: i64,
}
impl<'de> serde::Deserialize<'de> for PopupmenuSelect {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PopupmenuSelect;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid PopupmenuSelect")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = PopupmenuSelect {
                    selected: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct TablineUpdate {
    pub current: Tabpage,
    pub tabs: Vec<TablineTab>,
    pub current_buffer: Buffer,
    pub buffers: Vec<TablineBuffer>,
}
impl<'de> serde::Deserialize<'de> for TablineUpdate {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = TablineUpdate;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid TablineUpdate")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = TablineUpdate {
                    current: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    tabs: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    current_buffer: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    buffers: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct CmdlineShow {
    pub content: Vec<CmdlineContent>,
    pub pos: i64,
    pub firstc: String,
    pub prompt: String,
    pub indent: i64,
    pub level: i64,
}
impl<'de> serde::Deserialize<'de> for CmdlineShow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CmdlineShow;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid CmdlineShow")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = CmdlineShow {
                    content: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    pos: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    firstc: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                    prompt: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3usize, &self))?,
                    indent: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4usize, &self))?,
                    level: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct CmdlinePos {
    pub pos: i64,
    pub level: i64,
}
impl<'de> serde::Deserialize<'de> for CmdlinePos {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CmdlinePos;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid CmdlinePos")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = CmdlinePos {
                    pos: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    level: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct CmdlineSpecialChar {
    pub c: String,
    pub shift: bool,
    pub level: i64,
}
impl<'de> serde::Deserialize<'de> for CmdlineSpecialChar {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CmdlineSpecialChar;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid CmdlineSpecialChar")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = CmdlineSpecialChar {
                    c: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    shift: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    level: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct CmdlineHide {
    pub level: i64,
}
impl<'de> serde::Deserialize<'de> for CmdlineHide {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CmdlineHide;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid CmdlineHide")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = CmdlineHide {
                    level: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct CmdlineBlockShow {
    pub lines: Vec<Vec<CmdlineContent>>,
}
impl<'de> serde::Deserialize<'de> for CmdlineBlockShow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CmdlineBlockShow;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid CmdlineBlockShow")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = CmdlineBlockShow {
                    lines: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct CmdlineBlockAppend {
    pub lines: Vec<CmdlineContent>,
}
impl<'de> serde::Deserialize<'de> for CmdlineBlockAppend {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CmdlineBlockAppend;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid CmdlineBlockAppend")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = CmdlineBlockAppend {
                    lines: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WildmenuShow {
    pub items: Vec<rmpv::Value>,
}
impl<'de> serde::Deserialize<'de> for WildmenuShow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WildmenuShow;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WildmenuShow")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WildmenuShow {
                    items: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct WildmenuSelect {
    pub selected: i64,
}
impl<'de> serde::Deserialize<'de> for WildmenuSelect {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = WildmenuSelect;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid WildmenuSelect")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = WildmenuSelect {
                    selected: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct MsgShow {
    pub kind: String,
    pub content: Vec<MsgShowContent>,
    pub replace_last: bool,
}
impl<'de> serde::Deserialize<'de> for MsgShow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MsgShow;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid MsgShow")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = MsgShow {
                    kind: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                    content: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1usize, &self))?,
                    replace_last: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct MsgShowcmd {
    pub content: Vec<rmpv::Value>,
}
impl<'de> serde::Deserialize<'de> for MsgShowcmd {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MsgShowcmd;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid MsgShowcmd")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = MsgShowcmd {
                    content: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct MsgShowmode {
    pub content: Vec<rmpv::Value>,
}
impl<'de> serde::Deserialize<'de> for MsgShowmode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MsgShowmode;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid MsgShowmode")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = MsgShowmode {
                    content: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct MsgRuler {
    pub content: Vec<rmpv::Value>,
}
impl<'de> serde::Deserialize<'de> for MsgRuler {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MsgRuler;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid MsgRuler")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = MsgRuler {
                    content: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct MsgHistoryShow {
    pub entries: Vec<MsgHistoryShowEntry>,
}
impl<'de> serde::Deserialize<'de> for MsgHistoryShow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MsgHistoryShow;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid MsgHistoryShow")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = MsgHistoryShow {
                    entries: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub struct ErrorExit {
    pub status: i64,
}
impl<'de> serde::Deserialize<'de> for ErrorExit {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ErrorExit;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid ErrorExit")
            }
            fn visit_seq<V: serde::de::SeqAccess<'de>>(
                self,
                mut seq: V,
            ) -> Result<Self::Value, V::Error> {
                let evt = ErrorExit {
                    status: seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?,
                };
                while let Some(serde::de::IgnoredAny) = seq.next_element()? {}
                Ok(evt)
            }
        }
        d.deserialize_any(Visitor)
    }
}
#[derive(Debug)]
pub enum UiEvent {
    ModeInfoSet(Vec<ModeInfoSet>),
    UpdateMenu,
    BusyStart,
    BusyStop,
    MouseOn,
    MouseOff,
    ModeChange(Vec<ModeChange>),
    Bell,
    VisualBell,
    Flush,
    Suspend,
    SetTitle(Vec<SetTitle>),
    SetIcon(Vec<SetIcon>),
    Screenshot(Vec<Screenshot>),
    OptionSet(Vec<OptionSet>),
    Chdir(Vec<Chdir>),
    UpdateFg(Vec<UpdateFg>),
    UpdateBg(Vec<UpdateBg>),
    UpdateSp(Vec<UpdateSp>),
    Resize(Vec<Resize>),
    Clear,
    EolClear,
    CursorGoto(Vec<CursorGoto>),
    HighlightSet(Vec<HighlightSet>),
    Put(Vec<Put>),
    SetScrollRegion(Vec<SetScrollRegion>),
    Scroll(Vec<Scroll>),
    DefaultColorsSet(Vec<DefaultColorsSet>),
    HlAttrDefine(Vec<HlAttrDefine>),
    HlGroupSet(Vec<HlGroupSet>),
    GridResize(Vec<GridResize>),
    GridClear(Vec<GridClear>),
    GridCursorGoto(Vec<GridCursorGoto>),
    GridLine(Vec<GridLine>),
    GridScroll(Vec<GridScroll>),
    GridDestroy(Vec<GridDestroy>),
    WinPos(Vec<WinPos>),
    WinFloatPos(Vec<WinFloatPos>),
    WinExternalPos(Vec<WinExternalPos>),
    WinHide(Vec<WinHide>),
    WinClose(Vec<WinClose>),
    MsgSetPos(Vec<MsgSetPos>),
    WinViewport(Vec<WinViewport>),
    WinViewportMargins(Vec<WinViewportMargins>),
    WinExtmark(Vec<WinExtmark>),
    PopupmenuShow(Vec<PopupmenuShow>),
    PopupmenuHide,
    PopupmenuSelect(Vec<PopupmenuSelect>),
    TablineUpdate(Vec<TablineUpdate>),
    CmdlineShow(Vec<CmdlineShow>),
    CmdlinePos(Vec<CmdlinePos>),
    CmdlineSpecialChar(Vec<CmdlineSpecialChar>),
    CmdlineHide(Vec<CmdlineHide>),
    CmdlineBlockShow(Vec<CmdlineBlockShow>),
    CmdlineBlockAppend(Vec<CmdlineBlockAppend>),
    CmdlineBlockHide,
    WildmenuShow(Vec<WildmenuShow>),
    WildmenuSelect(Vec<WildmenuSelect>),
    WildmenuHide,
    MsgShow(Vec<MsgShow>),
    MsgClear,
    MsgShowcmd(Vec<MsgShowcmd>),
    MsgShowmode(Vec<MsgShowmode>),
    MsgRuler(Vec<MsgRuler>),
    MsgHistoryShow(Vec<MsgHistoryShow>),
    MsgHistoryClear,
    ErrorExit(Vec<ErrorExit>),
}
impl Display for UiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModeInfoSet(_) => write!(f, "mode_info_set"),
            Self::UpdateMenu => write!(f, "update_menu"),
            Self::BusyStart => write!(f, "busy_start"),
            Self::BusyStop => write!(f, "busy_stop"),
            Self::MouseOn => write!(f, "mouse_on"),
            Self::MouseOff => write!(f, "mouse_off"),
            Self::ModeChange(_) => write!(f, "mode_change"),
            Self::Bell => write!(f, "bell"),
            Self::VisualBell => write!(f, "visual_bell"),
            Self::Flush => write!(f, "flush"),
            Self::Suspend => write!(f, "suspend"),
            Self::SetTitle(_) => write!(f, "set_title"),
            Self::SetIcon(_) => write!(f, "set_icon"),
            Self::Screenshot(_) => write!(f, "screenshot"),
            Self::OptionSet(_) => write!(f, "option_set"),
            Self::Chdir(_) => write!(f, "chdir"),
            Self::UpdateFg(_) => write!(f, "update_fg"),
            Self::UpdateBg(_) => write!(f, "update_bg"),
            Self::UpdateSp(_) => write!(f, "update_sp"),
            Self::Resize(_) => write!(f, "resize"),
            Self::Clear => write!(f, "clear"),
            Self::EolClear => write!(f, "eol_clear"),
            Self::CursorGoto(_) => write!(f, "cursor_goto"),
            Self::HighlightSet(_) => write!(f, "highlight_set"),
            Self::Put(_) => write!(f, "put"),
            Self::SetScrollRegion(_) => write!(f, "set_scroll_region"),
            Self::Scroll(_) => write!(f, "scroll"),
            Self::DefaultColorsSet(_) => write!(f, "default_colors_set"),
            Self::HlAttrDefine(_) => write!(f, "hl_attr_define"),
            Self::HlGroupSet(_) => write!(f, "hl_group_set"),
            Self::GridResize(_) => write!(f, "grid_resize"),
            Self::GridClear(_) => write!(f, "grid_clear"),
            Self::GridCursorGoto(_) => write!(f, "grid_cursor_goto"),
            Self::GridLine(_) => write!(f, "grid_line"),
            Self::GridScroll(_) => write!(f, "grid_scroll"),
            Self::GridDestroy(_) => write!(f, "grid_destroy"),
            Self::WinPos(_) => write!(f, "win_pos"),
            Self::WinFloatPos(_) => write!(f, "win_float_pos"),
            Self::WinExternalPos(_) => write!(f, "win_external_pos"),
            Self::WinHide(_) => write!(f, "win_hide"),
            Self::WinClose(_) => write!(f, "win_close"),
            Self::MsgSetPos(_) => write!(f, "msg_set_pos"),
            Self::WinViewport(_) => write!(f, "win_viewport"),
            Self::WinViewportMargins(_) => write!(f, "win_viewport_margins"),
            Self::WinExtmark(_) => write!(f, "win_extmark"),
            Self::PopupmenuShow(_) => write!(f, "popupmenu_show"),
            Self::PopupmenuHide => write!(f, "popupmenu_hide"),
            Self::PopupmenuSelect(_) => write!(f, "popupmenu_select"),
            Self::TablineUpdate(_) => write!(f, "tabline_update"),
            Self::CmdlineShow(_) => write!(f, "cmdline_show"),
            Self::CmdlinePos(_) => write!(f, "cmdline_pos"),
            Self::CmdlineSpecialChar(_) => write!(f, "cmdline_special_char"),
            Self::CmdlineHide(_) => write!(f, "cmdline_hide"),
            Self::CmdlineBlockShow(_) => write!(f, "cmdline_block_show"),
            Self::CmdlineBlockAppend(_) => write!(f, "cmdline_block_append"),
            Self::CmdlineBlockHide => write!(f, "cmdline_block_hide"),
            Self::WildmenuShow(_) => write!(f, "wildmenu_show"),
            Self::WildmenuSelect(_) => write!(f, "wildmenu_select"),
            Self::WildmenuHide => write!(f, "wildmenu_hide"),
            Self::MsgShow(_) => write!(f, "msg_show"),
            Self::MsgClear => write!(f, "msg_clear"),
            Self::MsgShowcmd(_) => write!(f, "msg_showcmd"),
            Self::MsgShowmode(_) => write!(f, "msg_showmode"),
            Self::MsgRuler(_) => write!(f, "msg_ruler"),
            Self::MsgHistoryShow(_) => write!(f, "msg_history_show"),
            Self::MsgHistoryClear => write!(f, "msg_history_clear"),
            Self::ErrorExit(_) => write!(f, "error_exit"),
        }
    }
}
macro_rules! seq_to_vec {
    ($ seq : expr) => {{
        let mut v = Vec::with_capacity($seq.size_hint().unwrap_or(0));
        while let Some(evt) = $seq.next_element()? {
            v.push(evt);
        }
        v
    }};
}
impl<'de> serde::Deserialize<'de> for UiEvent {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UiEvent;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid UiEvent")
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                let name = seq
                    .next_element::<String>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                Ok(match name.as_str() {
                    "mode_info_set" => UiEvent::ModeInfoSet(seq_to_vec!(seq)),
                    "update_menu" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::UpdateMenu
                    }
                    "busy_start" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::BusyStart
                    }
                    "busy_stop" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::BusyStop
                    }
                    "mouse_on" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::MouseOn
                    }
                    "mouse_off" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::MouseOff
                    }
                    "mode_change" => UiEvent::ModeChange(seq_to_vec!(seq)),
                    "bell" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::Bell
                    }
                    "visual_bell" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::VisualBell
                    }
                    "flush" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::Flush
                    }
                    "suspend" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::Suspend
                    }
                    "set_title" => UiEvent::SetTitle(seq_to_vec!(seq)),
                    "set_icon" => UiEvent::SetIcon(seq_to_vec!(seq)),
                    "screenshot" => UiEvent::Screenshot(seq_to_vec!(seq)),
                    "option_set" => UiEvent::OptionSet(seq_to_vec!(seq)),
                    "chdir" => UiEvent::Chdir(seq_to_vec!(seq)),
                    "update_fg" => UiEvent::UpdateFg(seq_to_vec!(seq)),
                    "update_bg" => UiEvent::UpdateBg(seq_to_vec!(seq)),
                    "update_sp" => UiEvent::UpdateSp(seq_to_vec!(seq)),
                    "resize" => UiEvent::Resize(seq_to_vec!(seq)),
                    "clear" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::Clear
                    }
                    "eol_clear" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::EolClear
                    }
                    "cursor_goto" => UiEvent::CursorGoto(seq_to_vec!(seq)),
                    "highlight_set" => UiEvent::HighlightSet(seq_to_vec!(seq)),
                    "put" => UiEvent::Put(seq_to_vec!(seq)),
                    "set_scroll_region" => UiEvent::SetScrollRegion(seq_to_vec!(seq)),
                    "scroll" => UiEvent::Scroll(seq_to_vec!(seq)),
                    "default_colors_set" => UiEvent::DefaultColorsSet(seq_to_vec!(seq)),
                    "hl_attr_define" => UiEvent::HlAttrDefine(seq_to_vec!(seq)),
                    "hl_group_set" => UiEvent::HlGroupSet(seq_to_vec!(seq)),
                    "grid_resize" => UiEvent::GridResize(seq_to_vec!(seq)),
                    "grid_clear" => UiEvent::GridClear(seq_to_vec!(seq)),
                    "grid_cursor_goto" => UiEvent::GridCursorGoto(seq_to_vec!(seq)),
                    "grid_line" => UiEvent::GridLine(seq_to_vec!(seq)),
                    "grid_scroll" => UiEvent::GridScroll(seq_to_vec!(seq)),
                    "grid_destroy" => UiEvent::GridDestroy(seq_to_vec!(seq)),
                    "win_pos" => UiEvent::WinPos(seq_to_vec!(seq)),
                    "win_float_pos" => UiEvent::WinFloatPos(seq_to_vec!(seq)),
                    "win_external_pos" => UiEvent::WinExternalPos(seq_to_vec!(seq)),
                    "win_hide" => UiEvent::WinHide(seq_to_vec!(seq)),
                    "win_close" => UiEvent::WinClose(seq_to_vec!(seq)),
                    "msg_set_pos" => UiEvent::MsgSetPos(seq_to_vec!(seq)),
                    "win_viewport" => UiEvent::WinViewport(seq_to_vec!(seq)),
                    "win_viewport_margins" => UiEvent::WinViewportMargins(seq_to_vec!(seq)),
                    "win_extmark" => UiEvent::WinExtmark(seq_to_vec!(seq)),
                    "popupmenu_show" => UiEvent::PopupmenuShow(seq_to_vec!(seq)),
                    "popupmenu_hide" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::PopupmenuHide
                    }
                    "popupmenu_select" => UiEvent::PopupmenuSelect(seq_to_vec!(seq)),
                    "tabline_update" => UiEvent::TablineUpdate(seq_to_vec!(seq)),
                    "cmdline_show" => UiEvent::CmdlineShow(seq_to_vec!(seq)),
                    "cmdline_pos" => UiEvent::CmdlinePos(seq_to_vec!(seq)),
                    "cmdline_special_char" => UiEvent::CmdlineSpecialChar(seq_to_vec!(seq)),
                    "cmdline_hide" => UiEvent::CmdlineHide(seq_to_vec!(seq)),
                    "cmdline_block_show" => UiEvent::CmdlineBlockShow(seq_to_vec!(seq)),
                    "cmdline_block_append" => UiEvent::CmdlineBlockAppend(seq_to_vec!(seq)),
                    "cmdline_block_hide" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::CmdlineBlockHide
                    }
                    "wildmenu_show" => UiEvent::WildmenuShow(seq_to_vec!(seq)),
                    "wildmenu_select" => UiEvent::WildmenuSelect(seq_to_vec!(seq)),
                    "wildmenu_hide" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::WildmenuHide
                    }
                    "msg_show" => UiEvent::MsgShow(seq_to_vec!(seq)),
                    "msg_clear" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::MsgClear
                    }
                    "msg_showcmd" => UiEvent::MsgShowcmd(seq_to_vec!(seq)),
                    "msg_showmode" => UiEvent::MsgShowmode(seq_to_vec!(seq)),
                    "msg_ruler" => UiEvent::MsgRuler(seq_to_vec!(seq)),
                    "msg_history_show" => UiEvent::MsgHistoryShow(seq_to_vec!(seq)),
                    "msg_history_clear" => {
                        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
                        UiEvent::MsgHistoryClear
                    }
                    "error_exit" => UiEvent::ErrorExit(seq_to_vec!(seq)),
                    v => panic!("failed to decode message {:?}", v),
                })
            }
        }
        d.deserialize_seq(Visitor)
    }
}
