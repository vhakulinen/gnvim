#[derive(Debug, Default, into_value_proc::IntoValue)]
pub struct UiOptions {
    pub rgb: bool,
    pub r#override: bool,
    pub ext_cmdline: bool,
    pub ext_hlstate: bool,
    pub ext_linegrid: bool,
    pub ext_messages: bool,
    pub ext_multigrid: bool,
    pub ext_popupmenu: bool,
    pub ext_tabline: bool,
    pub ext_termcolors: bool,
}
