macro_rules! args {
    ( $( $x:expr ),* ) => {
        {
            vec!(
                Value::Array(vec!(
                    $(
                        $x,
                    )*
                ))
            )
        }
    }
}

mod parse_redraw_event_tests {

    use neovim_lib::neovim_api::Tabpage;
    use neovim_lib::Value;
    use nvim_bridge;
    use nvim_bridge::{
        Cell, CmdlineBlockAppend, CmdlinePos, CmdlineShow, CmdlineSpecialChar,
        CompletionItem, CompletionItemKind, CursorShape, DefaultColorsSet,
        GridCursorGoto, GridLineSegment, GridResize, GridScroll, HlAttrDefine,
        ModeChange, ModeInfo, ModeInfoSet, OptionSet, PopupmenuShow,
        RedrawEvent, TablineUpdate, WildmenuShow,
    };
    use ui::color::{Color, Highlight};

    #[test]
    fn set_title() {
        let expected =
            vec![RedrawEvent::SetTitle(vec!["my title".to_string()])];

        let res = nvim_bridge::parse_redraw_event(args!(
            String::from("set_title").into(),
            Value::Array(vec!(String::from("my title").into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn grid_line() {
        let expected = vec![RedrawEvent::GridLine(vec![
            GridLineSegment {
                grid: 1,
                row: 1,
                col_start: 4,
                cells: vec![
                    Cell {
                        hl_id: 1,
                        repeat: 4,
                        text: " ".to_owned(),
                        double_width: false,
                    },
                    Cell {
                        hl_id: 4,
                        repeat: 1,
                        text: "3".to_owned(),
                        double_width: false,
                    },
                    Cell {
                        hl_id: 4,
                        repeat: 1,
                        text: "3".to_owned(),
                        double_width: true,
                    },
                    Cell {
                        hl_id: 1,
                        repeat: 1,
                        text: "".to_owned(),
                        double_width: false,
                    },
                ],
            },
            GridLineSegment {
                grid: 2,
                row: 4,
                col_start: 1,
                cells: vec![
                    Cell {
                        hl_id: 3,
                        repeat: 2,
                        text: "i".to_owned(),
                        double_width: false,
                    },
                    Cell {
                        hl_id: 1,
                        repeat: 1,
                        text: "2".to_owned(),
                        double_width: false,
                    },
                ],
            },
        ])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "grid_line".into(),
            Value::Array(vec!(
                1.into(),
                1.into(),
                4.into(),
                Value::Array(vec!(
                    Value::Array(vec!(" ".into(), 1.into(), 4.into())),
                    Value::Array(vec!("3".into(), 4.into())),
                    Value::Array(vec!("3".into())),
                    Value::Array(vec!("".into(), 1.into())),
                )),
            )),
            Value::Array(vec!(
                2.into(),
                4.into(),
                1.into(),
                Value::Array(vec!(
                    Value::Array(vec!("i".into(), 3.into(), 2.into())),
                    Value::Array(vec!("2".into(), 1.into())),
                )),
            ))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn grid_cursor_goto() {
        let expected =
            vec![RedrawEvent::GridCursorGoto(vec![GridCursorGoto {
                grid: 123,
                row: 321,
                col: 2,
            }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            String::from("grid_cursor_goto").into(),
            Value::Array(vec!(123.into(), 321.into(), 2.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn grid_resize() {
        let expected = vec![RedrawEvent::GridResize(vec![GridResize {
            grid: 2,
            width: 32,
            height: 12,
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "grid_resize".into(),
            Value::Array(vec!(2.into(), 32.into(), 12.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn grid_clear() {
        let expected = vec![RedrawEvent::GridClear(vec![32])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "grid_clear".into(),
            Value::Array(vec!(32.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn grid_scroll() {
        let expected = vec![RedrawEvent::GridScroll(vec![GridScroll {
            grid: 1,
            reg: [132, 321, 2, 51],
            rows: 12,
            cols: 32,
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "grid_scroll".into(),
            Value::Array(vec!(
                1.into(),
                132.into(),
                321.into(),
                2.into(),
                51.into(),
                12.into(),
                32.into(),
            ))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn default_colors_set() {
        let expected =
            vec![RedrawEvent::DefaultColorsSet(vec![DefaultColorsSet {
                fg: Color::from_u64(321921),
                bg: Color::from_u64(94921),
                sp: Color::from_u64(983821232),
            }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "default_colors_set".into(),
            Value::Array(vec!(321921.into(), 94921.into(), 983821232.into(),))
        ));

        assert_eq!(expected, res);
    }

    /// Test default values.
    #[test]
    fn default_colors_set2() {
        let expected =
            vec![RedrawEvent::DefaultColorsSet(vec![DefaultColorsSet {
                fg: Color::from_u64(0),
                bg: Color::from_u64(std::u64::MAX),
                sp: Color::from_u64(16711680),
            }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "default_colors_set".into(),
            Value::Array(vec!(
                (-1 as i64).into(),
                (-1 as i64).into(),
                (-1 as i64).into(),
            ))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn hl_attr_define() {
        let expected = vec![RedrawEvent::HlAttrDefine(vec![
            HlAttrDefine {
                id: 1,
                hl: Highlight {
                    foreground: Some(Color::from_u64(3215)),
                    background: Some(Color::from_u64(214)),
                    special: Some(Color::from_u64(2019092)),
                    reverse: false,
                    italic: true,
                    bold: true,
                    underline: true,
                    undercurl: false,
                },
            },
            HlAttrDefine {
                id: 42,
                hl: Highlight {
                    foreground: Some(Color::from_u64(3215)),
                    background: None,
                    special: Some(Color::from_u64(2019092)),
                    reverse: true,
                    italic: false,
                    bold: true,
                    underline: false,
                    undercurl: true,
                },
            },
            HlAttrDefine {
                id: 32,
                hl: Highlight {
                    foreground: Some(Color::from_u64(215)),
                    background: Some(Color::from_u64(315)),
                    special: Some(Color::from_u64(19092)),
                    reverse: true,
                    italic: true,
                    bold: true,
                    underline: false,
                    undercurl: true,
                },
            },
            HlAttrDefine {
                id: 3,
                hl: Highlight {
                    foreground: None,
                    background: None,
                    special: None,
                    reverse: false,
                    italic: false,
                    bold: false,
                    underline: false,
                    undercurl: false,
                },
            },
        ])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "hl_attr_define".into(),
            Value::Array(vec!(
                1.into(),
                Value::Map(vec!(
                    ("foreground".into(), 3215.into()),
                    ("background".into(), 214.into()),
                    ("special".into(), 2019092.into()),
                    ("reverse".into(), false.into()),
                    ("italic".into(), true.into()),
                    ("bold".into(), true.into()),
                    ("underline".into(), true.into()),
                )),
            )),
            Value::Array(vec!(
                42.into(),
                Value::Map(vec!(
                    ("foreground".into(), 3215.into()),
                    ("special".into(), 2019092.into()),
                    ("reverse".into(), true.into()),
                    ("bold".into(), true.into()),
                    ("undercurl".into(), true.into()),
                )),
            )),
            Value::Array(vec!(
                32.into(),
                Value::Map(vec!(
                    ("foreground".into(), 215.into()),
                    ("background".into(), 315.into()),
                    ("special".into(), 19092.into()),
                    ("reverse".into(), true.into()),
                    ("italic".into(), true.into()),
                    ("bold".into(), true.into()),
                    ("undercurl".into(), true.into()),
                )),
            )),
            Value::Array(vec!(3.into(), Value::Map(vec!()),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn option_set() {
        let expected = vec![RedrawEvent::OptionSet(vec![
            OptionSet::GuiFont("my awesome font:h32".into()),
            OptionSet::LineSpace(32),
        ])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "option_set".into(),
            Value::Array(vec!("guifont".into(), "my awesome font:h32".into(),)),
            Value::Array(vec!("linespace".into(), 32.into()))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn mode_info_set() {
        let expected = vec![RedrawEvent::ModeInfoSet(vec![ModeInfoSet {
            cursor_shape_enabled: true,
            mode_info: vec![
                ModeInfo {
                    blink_on: 32,
                    cursor_shape: CursorShape::Horizontal,
                    cell_percentage: 0.32,
                },
                ModeInfo {
                    blink_on: 1,
                    cursor_shape: CursorShape::Block,
                    cell_percentage: 1.0,
                },
            ],
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "mode_info_set".into(),
            Value::Array(vec!(
                true.into(),
                Value::Array(vec!(
                    Value::Map(vec!(
                        ("blinkon".into(), 32.into()),
                        ("cursor_shape".into(), "horizontal".into()),
                        ("cell_percentage".into(), 32.into()),
                    )),
                    Value::Map(vec!(
                        ("blinkon".into(), 1.into()),
                        ("cursor_shape".into(), "block".into()),
                        ("cell_percentage".into(), 100.into()),
                    )),
                )),
            ))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn mode_change() {
        let expected = vec![RedrawEvent::ModeChange(vec![ModeChange {
            name: "foo".into(),
            index: 32,
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "mode_change".into(),
            Value::Array(vec!("foo".into(), 32.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn busy_start() {
        let expected = vec![RedrawEvent::SetBusy(true)];

        let res = nvim_bridge::parse_redraw_event(args!("busy_start".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn busy_stop() {
        let expected = vec![RedrawEvent::SetBusy(false)];

        let res = nvim_bridge::parse_redraw_event(args!("busy_stop".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn flush() {
        let expected = vec![RedrawEvent::Flush()];

        let res = nvim_bridge::parse_redraw_event(args!("flush".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn popupmenu_show() {
        let expected = vec![RedrawEvent::PopupmenuShow(vec![PopupmenuShow {
            selected: 4,
            row: 3,
            col: 6,
            items: vec![
                CompletionItem {
                    word: "foo".to_owned(),
                    kind: CompletionItemKind::Class,
                    kind_raw: "class".to_owned(),
                    menu: "bar".to_owned(),
                    info: "foobar321".to_owned(),
                },
                CompletionItem {
                    word: "drow".to_owned(),
                    kind: CompletionItemKind::Unknown,
                    kind_raw: "".to_owned(),
                    menu: "unem".to_owned(),
                    info: "ofni".to_owned(),
                },
            ],
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "popupmenu_show".into(),
            Value::Array(vec!(
                Value::Array(vec!(
                    Value::Array(vec!(
                        "foo".into(),
                        "class".into(),
                        "bar".into(),
                        "foobar321".into(),
                    )),
                    Value::Array(vec!(
                        "drow".into(),
                        "".into(),
                        "unem".into(),
                        "ofni".into(),
                    )),
                )),
                4.into(),
                3.into(),
                6.into(),
            ))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn popupmenu_hide() {
        let expected = vec![RedrawEvent::PopupmenuHide()];

        let res =
            nvim_bridge::parse_redraw_event(args!("popupmenu_hide".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn popupmenu_select() {
        let expected = vec![RedrawEvent::PopupmenuSelect(vec![32])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "popupmenu_select".into(),
            Value::Array(vec!(32.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn tabline_update() {
        let expected = vec![RedrawEvent::TablineUpdate(vec![TablineUpdate {
            current: Tabpage::new("foo".into()),
            tabs: vec![
                (Tabpage::new("bar".into()), "bar_name".into()),
                (Tabpage::new("ugh".into()), "ugh_name".into()),
            ],
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "tabline_update".into(),
            Value::Array(vec!(
                "foo".into(),
                Value::Array(vec!(
                    Value::Map(vec!(
                        ("tab".into(), "bar".into()),
                        ("name".into(), "bar_name".into()),
                    )),
                    Value::Map(vec!(
                        ("tab".into(), "ugh".into()),
                        ("name".into(), "ugh_name".into()),
                    )),
                )),
            ))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn cmdline_show() {
        let expected = vec![RedrawEvent::CmdlineShow(vec![CmdlineShow {
            content: vec![(91, "foo".to_owned()), (33, "bar".to_owned())],
            pos: 32,
            firstc: "f".to_owned(),
            prompt: "p".to_owned(),
            indent: 2,
            level: 4,
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "cmdline_show".into(),
            Value::Array(vec![
                Value::Array(vec![
                    Value::Array(vec![91.into(), "foo".into()]),
                    Value::Array(vec![33.into(), "bar".into()]),
                ]),
                32.into(),
                "f".into(),
                "p".into(),
                2.into(),
                4.into(),
            ])
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn cmdline_hide() {
        let expected = vec![RedrawEvent::CmdlineHide()];

        let res = nvim_bridge::parse_redraw_event(args!("cmdline_hide".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn cmdline_pos() {
        let expected = vec![RedrawEvent::CmdlinePos(vec![CmdlinePos {
            pos: 3,
            level: 9,
        }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "cmdline_pos".into(),
            Value::Array(vec!(3.into(), 9.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn cmdline_special_char() {
        let expected =
            vec![RedrawEvent::CmdlineSpecialChar(vec![CmdlineSpecialChar {
                character: "^V".to_string(),
                shift: false,
                level: 1,
            }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "cmdline_special_char".into(),
            Value::Array(vec!("^V".into(), false.into(), 1.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn cmdline_block_append() {
        let expected =
            vec![RedrawEvent::CmdlineBlockAppend(vec![CmdlineBlockAppend {
                line: vec![(2, "foobar".to_string()), (1, "bar".to_string())],
            }])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "cmdline_block_append".into(),
            Value::Array(vec!(Value::Array(vec!(
                Value::Array(vec!(2.into(), "foobar".into(),)),
                Value::Array(vec!(1.into(), "bar".into(),)),
            )),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn cmdline_block_hide() {
        let expected = vec![RedrawEvent::CmdlineBlockHide()];

        let res =
            nvim_bridge::parse_redraw_event(args!("cmdline_block_hide".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn wildmenu_show() {
        let expected =
            vec![RedrawEvent::WildmenuShow(vec![WildmenuShow(vec![
                "foo".to_owned(),
                "bar".to_owned(),
            ])])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "wildmenu_show".into(),
            Value::Array(
                vec!(Value::Array(vec!("foo".into(), "bar".into(),)),)
            )
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn wildmenu_hide() {
        let expected = vec![RedrawEvent::WildmenuHide()];

        let res =
            nvim_bridge::parse_redraw_event(args!("wildmenu_hide".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn wildmenu_select() {
        let expected = vec![RedrawEvent::WildmenuSelect(vec![32])];

        let res = nvim_bridge::parse_redraw_event(args!(
            "wildmenu_select".into(),
            Value::Array(vec!(32.into(),))
        ));

        assert_eq!(expected, res);
    }

    #[test]
    fn mouse_on() {
        let expected = vec![RedrawEvent::Ignored("mouse_on".to_owned())];

        let res = nvim_bridge::parse_redraw_event(args!("mouse_on".into()));

        assert_eq!(expected, res);
    }

    #[test]
    fn mouse_off() {
        let expected = vec![RedrawEvent::Ignored("mouse_off".to_owned())];

        let res = nvim_bridge::parse_redraw_event(args!("mouse_off".into()));

        assert_eq!(expected, res);
    }
}

mod parse_gnvim_event_tests {

    use neovim_lib::Value;
    use nvim_bridge;
    use nvim_bridge::{
        CmdlineColors, GnvimEvent, PmenuColors, SetGuiColors, TablineColors,
        WildmenuColors,
    };
    use ui::color::Color;

    #[test]
    fn set_gui_colors() {
        let expected: Result<GnvimEvent, String> =
            Ok(GnvimEvent::SetGuiColors(SetGuiColors {
                pmenu: PmenuColors {
                    bg: Some(
                        Color::from_hex_string("#00ff00".to_owned()).unwrap(),
                    ),
                    fg: Some(
                        Color::from_hex_string("#0000ff".to_owned()).unwrap(),
                    ),
                    sel_bg: Some(
                        Color::from_hex_string("#00ff00".to_owned()).unwrap(),
                    ),
                    sel_fg: Some(
                        Color::from_hex_string("#0000ff".to_owned()).unwrap(),
                    ),
                },
                tabline: TablineColors {
                    bg: Some(
                        Color::from_hex_string("#00ff00".to_owned()).unwrap(),
                    ),
                    fg: Some(
                        Color::from_hex_string("#0000ff".to_owned()).unwrap(),
                    ),
                    sel_bg: Some(
                        Color::from_hex_string("#00ff00".to_owned()).unwrap(),
                    ),
                    sel_fg: Some(
                        Color::from_hex_string("#0000ff".to_owned()).unwrap(),
                    ),
                    fill_bg: Some(
                        Color::from_hex_string("#f0ff00".to_owned()).unwrap(),
                    ),
                    fill_fg: Some(
                        Color::from_hex_string("#f000ff".to_owned()).unwrap(),
                    ),
                },
                cmdline: CmdlineColors {
                    bg: Some(
                        Color::from_hex_string("#0Aff00".to_owned()).unwrap(),
                    ),
                    fg: Some(
                        Color::from_hex_string("#0A00ff".to_owned()).unwrap(),
                    ),
                    border: Some(
                        Color::from_hex_string("#A0ff00".to_owned()).unwrap(),
                    ),
                },
                wildmenu: WildmenuColors {
                    bg: Some(
                        Color::from_hex_string("#00ffe0".to_owned()).unwrap(),
                    ),
                    fg: Some(
                        Color::from_hex_string("#0000ef".to_owned()).unwrap(),
                    ),
                    sel_bg: Some(
                        Color::from_hex_string("#e0ff00".to_owned()).unwrap(),
                    ),
                    sel_fg: Some(
                        Color::from_hex_string("#0e00ff".to_owned()).unwrap(),
                    ),
                },
            }));

        let res = nvim_bridge::parse_gnvim_event(vec![
            "SetGuiColors".into(),
            Value::Map(vec![
                ("pmenu_bg".into(), "#00ff00".into()),
                ("pmenu_fg".into(), "#0000ff".into()),
                ("pmenusel_bg".into(), "#00ff00".into()),
                ("pmenusel_fg".into(), "#0000ff".into()),
                ("tabline_bg".into(), "#00ff00".into()),
                ("tabline_fg".into(), "#0000ff".into()),
                ("tablinesel_bg".into(), "#00ff00".into()),
                ("tablinesel_fg".into(), "#0000ff".into()),
                ("tablinefill_bg".into(), "#f0ff00".into()),
                ("tablinefill_fg".into(), "#f000ff".into()),
                ("cmdline_bg".into(), "#0Aff00".into()),
                ("cmdline_fg".into(), "#0A00ff".into()),
                ("cmdline_border".into(), "#A0ff00".into()),
                ("wildmenu_bg".into(), "#00ffe0".into()),
                ("wildmenu_fg".into(), "#0000ef".into()),
                ("wildmenusel_bg".into(), "#e0ff00".into()),
                ("wildmenusel_fg".into(), "#0e00ff".into()),
            ]),
        ]);

        assert_eq!(expected, res);
    }

    #[test]
    fn completion_menu_toggle_info() {
        let expected: Result<GnvimEvent, String> =
            Ok(GnvimEvent::CompletionMenuToggleInfo);

        let res = nvim_bridge::parse_gnvim_event(vec![
            "CompletionMenuToggleInfo".into(),
        ]);

        assert_eq!(expected, res);
    }

    #[test]
    fn cursor_tooltip_load_style() {
        let expected: Result<GnvimEvent, String> =
            Ok(GnvimEvent::CursorTooltipLoadStyle("foobar".to_owned()));

        let res = nvim_bridge::parse_gnvim_event(vec![
            "CursorTooltipLoadStyle".into(),
            "foobar".into(),
        ]);

        assert_eq!(expected, res);
    }

    #[test]
    fn cursor_tooltip_show() {
        let expected: Result<GnvimEvent, String> =
            Ok(GnvimEvent::CursorTooltipShow("foobar".to_owned(), 3, 6));

        let res = nvim_bridge::parse_gnvim_event(vec![
            "CursorTooltipShow".into(),
            "foobar".into(),
            3.into(),
            6.into(),
        ]);

        assert_eq!(expected, res);
    }

    #[test]
    fn cursor_tooltip_hide() {
        let expected: Result<GnvimEvent, String> =
            Ok(GnvimEvent::CursorTooltipHide);

        let res =
            nvim_bridge::parse_gnvim_event(vec!["CursorTooltipHide".into()]);

        assert_eq!(expected, res);
    }

    #[test]
    fn popupmenu_set_width() {
        let expected: Result<GnvimEvent, String> =
            Ok(GnvimEvent::PopupmenuWidth(432));

        let res = nvim_bridge::parse_gnvim_event(vec![
            "PopupmenuSetWidth".into(),
            432.into(),
        ]);

        assert_eq!(expected, res);
    }

    #[test]
    fn popupmenu_set_width_details() {
        let expected: Result<GnvimEvent, String> =
            Ok(GnvimEvent::PopupmenuWidthDetails(929));

        let res = nvim_bridge::parse_gnvim_event(vec![
            "PopupmenuSetWidthDetails".into(),
            929.into(),
        ]);

        assert_eq!(expected, res);
    }
}
