use crate::rpc::WriteError;
use crate::{
    rpc::{CallResponse, Caller},
    types::{Buffer, Dictionary, LuaRef, Object, Tabpage, UiOptions, Window},
};
impl<T> Neovim for T where T: Caller {}
# [async_trait :: async_trait (? Send)]
pub trait Neovim
where
    Self: Caller,
{
    async fn nvim_get_autocmds(
        self,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_autocmds", (opts,)).await
    }
    async fn nvim_create_autocmd(
        self,
        event: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_create_autocmd", (event, opts)).await
    }
    async fn nvim_del_autocmd(self, id: i64) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_autocmd", (id,)).await
    }
    async fn nvim_clear_autocmds(self, opts: &Dictionary) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_clear_autocmds", (opts,)).await
    }
    async fn nvim_create_augroup(
        self,
        name: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_create_augroup", (name, opts)).await
    }
    async fn nvim_del_augroup_by_id(self, id: i64) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_augroup_by_id", (id,)).await
    }
    async fn nvim_del_augroup_by_name(self, name: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_augroup_by_name", (name,)).await
    }
    async fn nvim_exec_autocmds(
        self,
        event: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_exec_autocmds", (event, opts)).await
    }
    async fn nvim_buf_line_count(self, buffer: &Buffer) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_buf_line_count", (buffer,)).await
    }
    async fn nvim_buf_attach(
        self,
        buffer: &Buffer,
        send_buffer: bool,
        opts: &Dictionary,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_attach", (buffer, send_buffer, opts))
            .await
    }
    async fn nvim_buf_detach(self, buffer: &Buffer) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_detach", (buffer,)).await
    }
    async fn nvim_buf_get_lines(
        self,
        buffer: &Buffer,
        start: i64,
        end: i64,
        strict_indexing: bool,
    ) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call("nvim_buf_get_lines", (buffer, start, end, strict_indexing))
            .await
    }
    async fn nvim_buf_set_lines(
        self,
        buffer: &Buffer,
        start: i64,
        end: i64,
        strict_indexing: bool,
        replacement: Vec<String>,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_set_lines",
            (buffer, start, end, strict_indexing, replacement),
        )
        .await
    }
    async fn nvim_buf_set_text(
        self,
        buffer: &Buffer,
        start_row: i64,
        start_col: i64,
        end_row: i64,
        end_col: i64,
        replacement: Vec<String>,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_set_text",
            (buffer, start_row, start_col, end_row, end_col, replacement),
        )
        .await
    }
    async fn nvim_buf_get_text(
        self,
        buffer: &Buffer,
        start_row: i64,
        start_col: i64,
        end_row: i64,
        end_col: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call(
            "nvim_buf_get_text",
            (buffer, start_row, start_col, end_row, end_col, opts),
        )
        .await
    }
    async fn nvim_buf_get_offset(
        self,
        buffer: &Buffer,
        index: i64,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_buf_get_offset", (buffer, index)).await
    }
    async fn nvim_buf_get_var(
        self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_buf_get_var", (buffer, name)).await
    }
    async fn nvim_buf_get_changedtick(
        self,
        buffer: &Buffer,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_buf_get_changedtick", (buffer,)).await
    }
    async fn nvim_buf_get_keymap(
        self,
        buffer: &Buffer,
        mode: &str,
    ) -> Result<CallResponse<Vec<Dictionary>>, WriteError> {
        self.call("nvim_buf_get_keymap", (buffer, mode)).await
    }
    async fn nvim_buf_set_keymap(
        self,
        buffer: &Buffer,
        mode: &str,
        lhs: &str,
        rhs: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_keymap", (buffer, mode, lhs, rhs, opts))
            .await
    }
    async fn nvim_buf_del_keymap(
        self,
        buffer: &Buffer,
        mode: &str,
        lhs: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_del_keymap", (buffer, mode, lhs)).await
    }
    async fn nvim_buf_set_var(
        self,
        buffer: &Buffer,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_var", (buffer, name, value)).await
    }
    async fn nvim_buf_del_var(
        self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_del_var", (buffer, name)).await
    }
    async fn nvim_buf_get_name(self, buffer: &Buffer) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_buf_get_name", (buffer,)).await
    }
    async fn nvim_buf_set_name(
        self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_name", (buffer, name)).await
    }
    async fn nvim_buf_is_loaded(self, buffer: &Buffer) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_is_loaded", (buffer,)).await
    }
    async fn nvim_buf_delete(
        self,
        buffer: &Buffer,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_delete", (buffer, opts)).await
    }
    async fn nvim_buf_is_valid(self, buffer: &Buffer) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_is_valid", (buffer,)).await
    }
    async fn nvim_buf_del_mark(
        self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_del_mark", (buffer, name)).await
    }
    async fn nvim_buf_set_mark(
        self,
        buffer: &Buffer,
        name: &str,
        line: i64,
        col: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_set_mark", (buffer, name, line, col, opts))
            .await
    }
    async fn nvim_buf_get_mark(
        self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<(i64, i64)>, WriteError> {
        self.call("nvim_buf_get_mark", (buffer, name)).await
    }
    async fn nvim_buf_call(
        self,
        buffer: &Buffer,
        fun: &LuaRef,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_buf_call", (buffer, fun)).await
    }
    async fn nvim_parse_cmd(
        self,
        str: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_parse_cmd", (str, opts)).await
    }
    async fn nvim_cmd(
        self,
        cmd: &Dictionary,
        opts: &Dictionary,
    ) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_cmd", (cmd, opts)).await
    }
    async fn nvim_create_user_command(
        self,
        name: &str,
        command: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_create_user_command", (name, command, opts))
            .await
    }
    async fn nvim_del_user_command(self, name: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_user_command", (name,)).await
    }
    async fn nvim_buf_create_user_command(
        self,
        buffer: &Buffer,
        name: &str,
        command: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_create_user_command",
            (buffer, name, command, opts),
        )
        .await
    }
    async fn nvim_buf_del_user_command(
        self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_del_user_command", (buffer, name)).await
    }
    async fn nvim_get_commands(
        self,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_commands", (opts,)).await
    }
    async fn nvim_buf_get_commands(
        self,
        buffer: &Buffer,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_buf_get_commands", (buffer, opts)).await
    }
    async fn nvim_get_option_info(
        self,
        name: &str,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_option_info", (name,)).await
    }
    async fn nvim_create_namespace(self, name: &str) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_create_namespace", (name,)).await
    }
    async fn nvim_get_namespaces(self) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_namespaces", ()).await
    }
    async fn nvim_buf_get_extmark_by_id(
        self,
        buffer: &Buffer,
        ns_id: i64,
        id: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<i64>>, WriteError> {
        self.call("nvim_buf_get_extmark_by_id", (buffer, ns_id, id, opts))
            .await
    }
    async fn nvim_buf_get_extmarks(
        self,
        buffer: &Buffer,
        ns_id: i64,
        start: &Object,
        end: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_buf_get_extmarks", (buffer, ns_id, start, end, opts))
            .await
    }
    async fn nvim_buf_set_extmark(
        self,
        buffer: &Buffer,
        ns_id: i64,
        line: i64,
        col: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_buf_set_extmark", (buffer, ns_id, line, col, opts))
            .await
    }
    async fn nvim_buf_del_extmark(
        self,
        buffer: &Buffer,
        ns_id: i64,
        id: i64,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_del_extmark", (buffer, ns_id, id)).await
    }
    async fn nvim_buf_add_highlight(
        self,
        buffer: &Buffer,
        ns_id: i64,
        hl_group: &str,
        line: i64,
        col_start: i64,
        col_end: i64,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call(
            "nvim_buf_add_highlight",
            (buffer, ns_id, hl_group, line, col_start, col_end),
        )
        .await
    }
    async fn nvim_buf_clear_namespace(
        self,
        buffer: &Buffer,
        ns_id: i64,
        line_start: i64,
        line_end: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_clear_namespace",
            (buffer, ns_id, line_start, line_end),
        )
        .await
    }
    async fn nvim_set_decoration_provider(
        self,
        ns_id: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_decoration_provider", (ns_id, opts))
            .await
    }
    async fn nvim_get_option_value(
        self,
        name: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_option_value", (name, opts)).await
    }
    async fn nvim_set_option_value(
        self,
        name: &str,
        value: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_option_value", (name, value, opts))
            .await
    }
    async fn nvim_get_all_options_info(self) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_all_options_info", ()).await
    }
    async fn nvim_get_option_info2(
        self,
        name: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_option_info2", (name, opts)).await
    }
    async fn nvim_set_option(
        self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_option", (name, value)).await
    }
    async fn nvim_get_option(self, name: &str) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_option", (name,)).await
    }
    async fn nvim_buf_get_option(
        self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_buf_get_option", (buffer, name)).await
    }
    async fn nvim_buf_set_option(
        self,
        buffer: &Buffer,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_option", (buffer, name, value))
            .await
    }
    async fn nvim_win_get_option(
        self,
        window: &Window,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_win_get_option", (window, name)).await
    }
    async fn nvim_win_set_option(
        self,
        window: &Window,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_option", (window, name, value))
            .await
    }
    async fn nvim_tabpage_list_wins(
        self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<Vec<Window>>, WriteError> {
        self.call("nvim_tabpage_list_wins", (tabpage,)).await
    }
    async fn nvim_tabpage_get_var(
        self,
        tabpage: &Tabpage,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_tabpage_get_var", (tabpage, name)).await
    }
    async fn nvim_tabpage_set_var(
        self,
        tabpage: &Tabpage,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_tabpage_set_var", (tabpage, name, value))
            .await
    }
    async fn nvim_tabpage_del_var(
        self,
        tabpage: &Tabpage,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_tabpage_del_var", (tabpage, name)).await
    }
    async fn nvim_tabpage_get_win(
        self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<Window>, WriteError> {
        self.call("nvim_tabpage_get_win", (tabpage,)).await
    }
    async fn nvim_tabpage_get_number(
        self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_tabpage_get_number", (tabpage,)).await
    }
    async fn nvim_tabpage_is_valid(
        self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_tabpage_is_valid", (tabpage,)).await
    }
    async fn nvim_ui_attach(
        self,
        width: i64,
        height: i64,
        options: UiOptions,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_attach", (width, height, options)).await
    }
    async fn nvim_ui_set_focus(self, gained: bool) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_set_focus", (gained,)).await
    }
    async fn nvim_ui_detach(self) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_detach", ()).await
    }
    async fn nvim_ui_try_resize(
        self,
        width: i64,
        height: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_try_resize", (width, height)).await
    }
    async fn nvim_ui_set_option(
        self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_set_option", (name, value)).await
    }
    async fn nvim_ui_try_resize_grid(
        self,
        grid: i64,
        width: i64,
        height: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_try_resize_grid", (grid, width, height))
            .await
    }
    async fn nvim_ui_pum_set_height(self, height: i64) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_pum_set_height", (height,)).await
    }
    async fn nvim_ui_pum_set_bounds(
        self,
        width: f64,
        height: f64,
        row: f64,
        col: f64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_pum_set_bounds", (width, height, row, col))
            .await
    }
    async fn nvim_get_hl_id_by_name(self, name: &str) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_get_hl_id_by_name", (name,)).await
    }
    async fn nvim_get_hl(
        self,
        ns_id: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_hl", (ns_id, opts)).await
    }
    async fn nvim_set_hl(
        self,
        ns_id: i64,
        name: &str,
        val: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_hl", (ns_id, name, val)).await
    }
    async fn nvim_set_hl_ns(self, ns_id: i64) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_hl_ns", (ns_id,)).await
    }
    async fn nvim_set_hl_ns_fast(self, ns_id: i64) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_hl_ns_fast", (ns_id,)).await
    }
    async fn nvim_feedkeys(
        self,
        keys: &str,
        mode: &str,
        escape_ks: bool,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_feedkeys", (keys, mode, escape_ks)).await
    }
    async fn nvim_input(self, keys: &str) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_input", (keys,)).await
    }
    async fn nvim_input_mouse(
        self,
        button: &str,
        action: &str,
        modifier: &str,
        grid: i64,
        row: i64,
        col: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_input_mouse",
            (button, action, modifier, grid, row, col),
        )
        .await
    }
    async fn nvim_replace_termcodes(
        self,
        str: &str,
        from_part: bool,
        do_lt: bool,
        special: bool,
    ) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_replace_termcodes", (str, from_part, do_lt, special))
            .await
    }
    async fn nvim_exec_lua(
        self,
        code: &str,
        args: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_exec_lua", (code, args)).await
    }
    async fn nvim_notify(
        self,
        msg: &str,
        log_level: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_notify", (msg, log_level, opts)).await
    }
    async fn nvim_strwidth(self, text: &str) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_strwidth", (text,)).await
    }
    async fn nvim_list_runtime_paths(self) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call("nvim_list_runtime_paths", ()).await
    }
    async fn nvim_get_runtime_file(
        self,
        name: &str,
        all: bool,
    ) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call("nvim_get_runtime_file", (name, all)).await
    }
    async fn nvim_set_current_dir(self, dir: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_dir", (dir,)).await
    }
    async fn nvim_get_current_line(self) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_get_current_line", ()).await
    }
    async fn nvim_set_current_line(self, line: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_line", (line,)).await
    }
    async fn nvim_del_current_line(self) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_current_line", ()).await
    }
    async fn nvim_get_var(self, name: &str) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_var", (name,)).await
    }
    async fn nvim_set_var(
        self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_var", (name, value)).await
    }
    async fn nvim_del_var(self, name: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_var", (name,)).await
    }
    async fn nvim_get_vvar(self, name: &str) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_vvar", (name,)).await
    }
    async fn nvim_set_vvar(
        self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_vvar", (name, value)).await
    }
    async fn nvim_echo(
        self,
        chunks: Vec<rmpv::Value>,
        history: bool,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_echo", (chunks, history, opts)).await
    }
    async fn nvim_out_write(self, str: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_out_write", (str,)).await
    }
    async fn nvim_err_write(self, str: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_err_write", (str,)).await
    }
    async fn nvim_err_writeln(self, str: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_err_writeln", (str,)).await
    }
    async fn nvim_list_bufs(self) -> Result<CallResponse<Vec<Buffer>>, WriteError> {
        self.call("nvim_list_bufs", ()).await
    }
    async fn nvim_get_current_buf(self) -> Result<CallResponse<Buffer>, WriteError> {
        self.call("nvim_get_current_buf", ()).await
    }
    async fn nvim_set_current_buf(self, buffer: &Buffer) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_buf", (buffer,)).await
    }
    async fn nvim_list_wins(self) -> Result<CallResponse<Vec<Window>>, WriteError> {
        self.call("nvim_list_wins", ()).await
    }
    async fn nvim_get_current_win(self) -> Result<CallResponse<Window>, WriteError> {
        self.call("nvim_get_current_win", ()).await
    }
    async fn nvim_set_current_win(self, window: &Window) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_win", (window,)).await
    }
    async fn nvim_create_buf(
        self,
        listed: bool,
        scratch: bool,
    ) -> Result<CallResponse<Buffer>, WriteError> {
        self.call("nvim_create_buf", (listed, scratch)).await
    }
    async fn nvim_open_term(
        self,
        buffer: &Buffer,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_open_term", (buffer, opts)).await
    }
    async fn nvim_chan_send(self, chan: i64, data: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_chan_send", (chan, data)).await
    }
    async fn nvim_list_tabpages(self) -> Result<CallResponse<Vec<Tabpage>>, WriteError> {
        self.call("nvim_list_tabpages", ()).await
    }
    async fn nvim_get_current_tabpage(self) -> Result<CallResponse<Tabpage>, WriteError> {
        self.call("nvim_get_current_tabpage", ()).await
    }
    async fn nvim_set_current_tabpage(
        self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_tabpage", (tabpage,)).await
    }
    async fn nvim_paste(
        self,
        data: &str,
        crlf: bool,
        phase: i64,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_paste", (data, crlf, phase)).await
    }
    async fn nvim_put(
        self,
        lines: Vec<String>,
        _type: &str,
        after: bool,
        follow: bool,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_put", (lines, _type, after, follow)).await
    }
    async fn nvim_subscribe(self, event: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_subscribe", (event,)).await
    }
    async fn nvim_unsubscribe(self, event: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_unsubscribe", (event,)).await
    }
    async fn nvim_get_color_by_name(self, name: &str) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_get_color_by_name", (name,)).await
    }
    async fn nvim_get_color_map(self) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_color_map", ()).await
    }
    async fn nvim_get_context(
        self,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_context", (opts,)).await
    }
    async fn nvim_load_context(
        self,
        dict: &Dictionary,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_load_context", (dict,)).await
    }
    async fn nvim_get_mode(self) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_mode", ()).await
    }
    async fn nvim_get_keymap(
        self,
        mode: &str,
    ) -> Result<CallResponse<Vec<Dictionary>>, WriteError> {
        self.call("nvim_get_keymap", (mode,)).await
    }
    async fn nvim_set_keymap(
        self,
        mode: &str,
        lhs: &str,
        rhs: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_keymap", (mode, lhs, rhs, opts)).await
    }
    async fn nvim_del_keymap(self, mode: &str, lhs: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_keymap", (mode, lhs)).await
    }
    async fn nvim_get_api_info(self) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_api_info", ()).await
    }
    async fn nvim_set_client_info(
        self,
        name: &str,
        version: &Dictionary,
        _type: &str,
        methods: &Dictionary,
        attributes: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_set_client_info",
            (name, version, _type, methods, attributes),
        )
        .await
    }
    async fn nvim_get_chan_info(self, chan: i64) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_chan_info", (chan,)).await
    }
    async fn nvim_list_chans(self) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_list_chans", ()).await
    }
    async fn nvim_call_atomic(
        self,
        calls: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_call_atomic", (calls,)).await
    }
    async fn nvim_list_uis(self) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_list_uis", ()).await
    }
    async fn nvim_get_proc_children(
        self,
        pid: i64,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_proc_children", (pid,)).await
    }
    async fn nvim_get_proc(self, pid: i64) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_proc", (pid,)).await
    }
    async fn nvim_select_popupmenu_item(
        self,
        item: i64,
        insert: bool,
        finish: bool,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_select_popupmenu_item", (item, insert, finish, opts))
            .await
    }
    async fn nvim_del_mark(self, name: &str) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_del_mark", (name,)).await
    }
    async fn nvim_get_mark(
        self,
        name: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_mark", (name, opts)).await
    }
    async fn nvim_eval_statusline(
        self,
        str: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_eval_statusline", (str, opts)).await
    }
    async fn nvim_exec2(
        self,
        src: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_exec2", (src, opts)).await
    }
    async fn nvim_command(self, command: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_command", (command,)).await
    }
    async fn nvim_eval(self, expr: &str) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_eval", (expr,)).await
    }
    async fn nvim_call_function(
        self,
        _fn: &str,
        args: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_call_function", (_fn, args)).await
    }
    async fn nvim_call_dict_function(
        self,
        dict: &Object,
        _fn: &str,
        args: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_call_dict_function", (dict, _fn, args))
            .await
    }
    async fn nvim_parse_expression(
        self,
        expr: &str,
        flags: &str,
        highlight: bool,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_parse_expression", (expr, flags, highlight))
            .await
    }
    async fn nvim_open_win(
        self,
        buffer: &Buffer,
        enter: bool,
        config: &Dictionary,
    ) -> Result<CallResponse<Window>, WriteError> {
        self.call("nvim_open_win", (buffer, enter, config)).await
    }
    async fn nvim_win_set_config(
        self,
        window: &Window,
        config: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_config", (window, config)).await
    }
    async fn nvim_win_get_config(
        self,
        window: &Window,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_win_get_config", (window,)).await
    }
    async fn nvim_win_get_buf(self, window: &Window) -> Result<CallResponse<Buffer>, WriteError> {
        self.call("nvim_win_get_buf", (window,)).await
    }
    async fn nvim_win_set_buf(
        self,
        window: &Window,
        buffer: &Buffer,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_buf", (window, buffer)).await
    }
    async fn nvim_win_get_cursor(
        self,
        window: &Window,
    ) -> Result<CallResponse<(i64, i64)>, WriteError> {
        self.call("nvim_win_get_cursor", (window,)).await
    }
    async fn nvim_win_set_cursor(
        self,
        window: &Window,
        pos: (i64, i64),
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_cursor", (window, pos)).await
    }
    async fn nvim_win_get_height(self, window: &Window) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_win_get_height", (window,)).await
    }
    async fn nvim_win_set_height(
        self,
        window: &Window,
        height: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_height", (window, height)).await
    }
    async fn nvim_win_get_width(self, window: &Window) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_win_get_width", (window,)).await
    }
    async fn nvim_win_set_width(
        self,
        window: &Window,
        width: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_width", (window, width)).await
    }
    async fn nvim_win_get_var(
        self,
        window: &Window,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_win_get_var", (window, name)).await
    }
    async fn nvim_win_set_var(
        self,
        window: &Window,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_var", (window, name, value)).await
    }
    async fn nvim_win_del_var(
        self,
        window: &Window,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_del_var", (window, name)).await
    }
    async fn nvim_win_get_position(
        self,
        window: &Window,
    ) -> Result<CallResponse<(i64, i64)>, WriteError> {
        self.call("nvim_win_get_position", (window,)).await
    }
    async fn nvim_win_get_tabpage(
        self,
        window: &Window,
    ) -> Result<CallResponse<Tabpage>, WriteError> {
        self.call("nvim_win_get_tabpage", (window,)).await
    }
    async fn nvim_win_get_number(self, window: &Window) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_win_get_number", (window,)).await
    }
    async fn nvim_win_is_valid(self, window: &Window) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_win_is_valid", (window,)).await
    }
    async fn nvim_win_hide(self, window: &Window) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_hide", (window,)).await
    }
    async fn nvim_win_close(
        self,
        window: &Window,
        force: bool,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_close", (window, force)).await
    }
    async fn nvim_win_call(
        self,
        window: &Window,
        fun: &LuaRef,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_win_call", (window, fun)).await
    }
    async fn nvim_win_set_hl_ns(
        self,
        window: &Window,
        ns_id: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_hl_ns", (window, ns_id)).await
    }
}
