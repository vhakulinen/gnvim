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
    async fn nvim_get_autocmds(self, opts: &Dictionary) -> CallResponse<Vec<rmpv::Value>> {
        self.call("nvim_get_autocmds", (opts,)).await
    }
    async fn nvim_create_autocmd<T1: serde::Serialize>(
        self,
        event: &T1,
        opts: &Dictionary,
    ) -> CallResponse<i64> {
        self.call("nvim_create_autocmd", (event, opts)).await
    }
    async fn nvim_del_autocmd(self, id: i64) -> CallResponse<()> {
        self.call("nvim_del_autocmd", (id,)).await
    }
    async fn nvim_clear_autocmds(self, opts: &Dictionary) -> CallResponse<()> {
        self.call("nvim_clear_autocmds", (opts,)).await
    }
    async fn nvim_create_augroup(self, name: &str, opts: &Dictionary) -> CallResponse<i64> {
        self.call("nvim_create_augroup", (name, opts)).await
    }
    async fn nvim_del_augroup_by_id(self, id: i64) -> CallResponse<()> {
        self.call("nvim_del_augroup_by_id", (id,)).await
    }
    async fn nvim_del_augroup_by_name(self, name: &str) -> CallResponse<()> {
        self.call("nvim_del_augroup_by_name", (name,)).await
    }
    async fn nvim_exec_autocmds<T1: serde::Serialize>(
        self,
        event: &T1,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call("nvim_exec_autocmds", (event, opts)).await
    }
    async fn nvim_buf_line_count(self, buffer: &Buffer) -> CallResponse<i64> {
        self.call("nvim_buf_line_count", (buffer,)).await
    }
    async fn nvim_buf_attach(
        self,
        buffer: &Buffer,
        send_buffer: bool,
        opts: &Dictionary,
    ) -> CallResponse<bool> {
        self.call("nvim_buf_attach", (buffer, send_buffer, opts))
            .await
    }
    async fn nvim_buf_detach(self, buffer: &Buffer) -> CallResponse<bool> {
        self.call("nvim_buf_detach", (buffer,)).await
    }
    async fn nvim_buf_get_lines(
        self,
        buffer: &Buffer,
        start: i64,
        end: i64,
        strict_indexing: bool,
    ) -> CallResponse<Vec<String>> {
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
    ) -> CallResponse<()> {
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
    ) -> CallResponse<()> {
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
    ) -> CallResponse<Vec<String>> {
        self.call(
            "nvim_buf_get_text",
            (buffer, start_row, start_col, end_row, end_col, opts),
        )
        .await
    }
    async fn nvim_buf_get_offset(self, buffer: &Buffer, index: i64) -> CallResponse<i64> {
        self.call("nvim_buf_get_offset", (buffer, index)).await
    }
    async fn nvim_buf_get_var(self, buffer: &Buffer, name: &str) -> CallResponse<Object> {
        self.call("nvim_buf_get_var", (buffer, name)).await
    }
    async fn nvim_buf_get_changedtick(self, buffer: &Buffer) -> CallResponse<i64> {
        self.call("nvim_buf_get_changedtick", (buffer,)).await
    }
    async fn nvim_buf_get_keymap(
        self,
        buffer: &Buffer,
        mode: &str,
    ) -> CallResponse<Vec<Dictionary>> {
        self.call("nvim_buf_get_keymap", (buffer, mode)).await
    }
    async fn nvim_buf_set_keymap(
        self,
        buffer: &Buffer,
        mode: &str,
        lhs: &str,
        rhs: &str,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call("nvim_buf_set_keymap", (buffer, mode, lhs, rhs, opts))
            .await
    }
    async fn nvim_buf_del_keymap(self, buffer: &Buffer, mode: &str, lhs: &str) -> CallResponse<()> {
        self.call("nvim_buf_del_keymap", (buffer, mode, lhs)).await
    }
    async fn nvim_buf_set_var<T3: serde::Serialize>(
        self,
        buffer: &Buffer,
        name: &str,
        value: &T3,
    ) -> CallResponse<()> {
        self.call("nvim_buf_set_var", (buffer, name, value)).await
    }
    async fn nvim_buf_del_var(self, buffer: &Buffer, name: &str) -> CallResponse<()> {
        self.call("nvim_buf_del_var", (buffer, name)).await
    }
    async fn nvim_buf_get_name(self, buffer: &Buffer) -> CallResponse<String> {
        self.call("nvim_buf_get_name", (buffer,)).await
    }
    async fn nvim_buf_set_name(self, buffer: &Buffer, name: &str) -> CallResponse<()> {
        self.call("nvim_buf_set_name", (buffer, name)).await
    }
    async fn nvim_buf_is_loaded(self, buffer: &Buffer) -> CallResponse<bool> {
        self.call("nvim_buf_is_loaded", (buffer,)).await
    }
    async fn nvim_buf_delete(self, buffer: &Buffer, opts: &Dictionary) -> CallResponse<()> {
        self.call("nvim_buf_delete", (buffer, opts)).await
    }
    async fn nvim_buf_is_valid(self, buffer: &Buffer) -> CallResponse<bool> {
        self.call("nvim_buf_is_valid", (buffer,)).await
    }
    async fn nvim_buf_del_mark(self, buffer: &Buffer, name: &str) -> CallResponse<bool> {
        self.call("nvim_buf_del_mark", (buffer, name)).await
    }
    async fn nvim_buf_set_mark(
        self,
        buffer: &Buffer,
        name: &str,
        line: i64,
        col: i64,
        opts: &Dictionary,
    ) -> CallResponse<bool> {
        self.call("nvim_buf_set_mark", (buffer, name, line, col, opts))
            .await
    }
    async fn nvim_buf_get_mark(self, buffer: &Buffer, name: &str) -> CallResponse<(i64, i64)> {
        self.call("nvim_buf_get_mark", (buffer, name)).await
    }
    async fn nvim_buf_call(self, buffer: &Buffer, fun: &LuaRef) -> CallResponse<Object> {
        self.call("nvim_buf_call", (buffer, fun)).await
    }
    async fn nvim_parse_cmd(self, str: &str, opts: &Dictionary) -> CallResponse<Dictionary> {
        self.call("nvim_parse_cmd", (str, opts)).await
    }
    async fn nvim_cmd(self, cmd: &Dictionary, opts: &Dictionary) -> CallResponse<String> {
        self.call("nvim_cmd", (cmd, opts)).await
    }
    async fn nvim_create_user_command<T2: serde::Serialize>(
        self,
        name: &str,
        command: &T2,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call("nvim_create_user_command", (name, command, opts))
            .await
    }
    async fn nvim_del_user_command(self, name: &str) -> CallResponse<()> {
        self.call("nvim_del_user_command", (name,)).await
    }
    async fn nvim_buf_create_user_command<T3: serde::Serialize>(
        self,
        buffer: &Buffer,
        name: &str,
        command: &T3,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call(
            "nvim_buf_create_user_command",
            (buffer, name, command, opts),
        )
        .await
    }
    async fn nvim_buf_del_user_command(self, buffer: &Buffer, name: &str) -> CallResponse<()> {
        self.call("nvim_buf_del_user_command", (buffer, name)).await
    }
    async fn nvim_get_commands(self, opts: &Dictionary) -> CallResponse<Dictionary> {
        self.call("nvim_get_commands", (opts,)).await
    }
    async fn nvim_buf_get_commands(
        self,
        buffer: &Buffer,
        opts: &Dictionary,
    ) -> CallResponse<Dictionary> {
        self.call("nvim_buf_get_commands", (buffer, opts)).await
    }
    async fn nvim_get_option_info(self, name: &str) -> CallResponse<Dictionary> {
        self.call("nvim_get_option_info", (name,)).await
    }
    async fn nvim_create_namespace(self, name: &str) -> CallResponse<i64> {
        self.call("nvim_create_namespace", (name,)).await
    }
    async fn nvim_get_namespaces(self) -> CallResponse<Dictionary> {
        self.call("nvim_get_namespaces", ()).await
    }
    async fn nvim_buf_get_extmark_by_id(
        self,
        buffer: &Buffer,
        ns_id: i64,
        id: i64,
        opts: &Dictionary,
    ) -> CallResponse<Vec<i64>> {
        self.call("nvim_buf_get_extmark_by_id", (buffer, ns_id, id, opts))
            .await
    }
    async fn nvim_buf_get_extmarks<T3: serde::Serialize, T4: serde::Serialize>(
        self,
        buffer: &Buffer,
        ns_id: i64,
        start: &T3,
        end: &T4,
        opts: &Dictionary,
    ) -> CallResponse<Vec<rmpv::Value>> {
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
    ) -> CallResponse<i64> {
        self.call("nvim_buf_set_extmark", (buffer, ns_id, line, col, opts))
            .await
    }
    async fn nvim_buf_del_extmark(
        self,
        buffer: &Buffer,
        ns_id: i64,
        id: i64,
    ) -> CallResponse<bool> {
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
    ) -> CallResponse<i64> {
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
    ) -> CallResponse<()> {
        self.call(
            "nvim_buf_clear_namespace",
            (buffer, ns_id, line_start, line_end),
        )
        .await
    }
    async fn nvim_set_decoration_provider(self, ns_id: i64, opts: &Dictionary) -> CallResponse<()> {
        self.call("nvim_set_decoration_provider", (ns_id, opts))
            .await
    }
    async fn nvim_get_option_value(self, name: &str, opts: &Dictionary) -> CallResponse<Object> {
        self.call("nvim_get_option_value", (name, opts)).await
    }
    async fn nvim_set_option_value<T2: serde::Serialize>(
        self,
        name: &str,
        value: &T2,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call("nvim_set_option_value", (name, value, opts))
            .await
    }
    async fn nvim_get_all_options_info(self) -> CallResponse<Dictionary> {
        self.call("nvim_get_all_options_info", ()).await
    }
    async fn nvim_get_option_info2(
        self,
        name: &str,
        opts: &Dictionary,
    ) -> CallResponse<Dictionary> {
        self.call("nvim_get_option_info2", (name, opts)).await
    }
    async fn nvim_set_option<T2: serde::Serialize>(
        self,
        name: &str,
        value: &T2,
    ) -> CallResponse<()> {
        self.call("nvim_set_option", (name, value)).await
    }
    async fn nvim_get_option(self, name: &str) -> CallResponse<Object> {
        self.call("nvim_get_option", (name,)).await
    }
    async fn nvim_buf_get_option(self, buffer: &Buffer, name: &str) -> CallResponse<Object> {
        self.call("nvim_buf_get_option", (buffer, name)).await
    }
    async fn nvim_buf_set_option<T3: serde::Serialize>(
        self,
        buffer: &Buffer,
        name: &str,
        value: &T3,
    ) -> CallResponse<()> {
        self.call("nvim_buf_set_option", (buffer, name, value))
            .await
    }
    async fn nvim_win_get_option(self, window: &Window, name: &str) -> CallResponse<Object> {
        self.call("nvim_win_get_option", (window, name)).await
    }
    async fn nvim_win_set_option<T3: serde::Serialize>(
        self,
        window: &Window,
        name: &str,
        value: &T3,
    ) -> CallResponse<()> {
        self.call("nvim_win_set_option", (window, name, value))
            .await
    }
    async fn nvim_tabpage_list_wins(self, tabpage: &Tabpage) -> CallResponse<Vec<Window>> {
        self.call("nvim_tabpage_list_wins", (tabpage,)).await
    }
    async fn nvim_tabpage_get_var(self, tabpage: &Tabpage, name: &str) -> CallResponse<Object> {
        self.call("nvim_tabpage_get_var", (tabpage, name)).await
    }
    async fn nvim_tabpage_set_var<T3: serde::Serialize>(
        self,
        tabpage: &Tabpage,
        name: &str,
        value: &T3,
    ) -> CallResponse<()> {
        self.call("nvim_tabpage_set_var", (tabpage, name, value))
            .await
    }
    async fn nvim_tabpage_del_var(self, tabpage: &Tabpage, name: &str) -> CallResponse<()> {
        self.call("nvim_tabpage_del_var", (tabpage, name)).await
    }
    async fn nvim_tabpage_get_win(self, tabpage: &Tabpage) -> CallResponse<Window> {
        self.call("nvim_tabpage_get_win", (tabpage,)).await
    }
    async fn nvim_tabpage_get_number(self, tabpage: &Tabpage) -> CallResponse<i64> {
        self.call("nvim_tabpage_get_number", (tabpage,)).await
    }
    async fn nvim_tabpage_is_valid(self, tabpage: &Tabpage) -> CallResponse<bool> {
        self.call("nvim_tabpage_is_valid", (tabpage,)).await
    }
    async fn nvim_ui_attach(self, width: i64, height: i64, options: UiOptions) -> CallResponse<()> {
        self.call("nvim_ui_attach", (width, height, options)).await
    }
    async fn nvim_ui_set_focus(self, gained: bool) -> CallResponse<()> {
        self.call("nvim_ui_set_focus", (gained,)).await
    }
    async fn nvim_ui_detach(self) -> CallResponse<()> {
        self.call("nvim_ui_detach", ()).await
    }
    async fn nvim_ui_try_resize(self, width: i64, height: i64) -> CallResponse<()> {
        self.call("nvim_ui_try_resize", (width, height)).await
    }
    async fn nvim_ui_set_option<T2: serde::Serialize>(
        self,
        name: &str,
        value: &T2,
    ) -> CallResponse<()> {
        self.call("nvim_ui_set_option", (name, value)).await
    }
    async fn nvim_ui_try_resize_grid(self, grid: i64, width: i64, height: i64) -> CallResponse<()> {
        self.call("nvim_ui_try_resize_grid", (grid, width, height))
            .await
    }
    async fn nvim_ui_pum_set_height(self, height: i64) -> CallResponse<()> {
        self.call("nvim_ui_pum_set_height", (height,)).await
    }
    async fn nvim_ui_pum_set_bounds(
        self,
        width: f64,
        height: f64,
        row: f64,
        col: f64,
    ) -> CallResponse<()> {
        self.call("nvim_ui_pum_set_bounds", (width, height, row, col))
            .await
    }
    async fn nvim_get_hl_id_by_name(self, name: &str) -> CallResponse<i64> {
        self.call("nvim_get_hl_id_by_name", (name,)).await
    }
    async fn nvim_get_hl(self, ns_id: i64, opts: &Dictionary) -> CallResponse<Dictionary> {
        self.call("nvim_get_hl", (ns_id, opts)).await
    }
    async fn nvim_set_hl(self, ns_id: i64, name: &str, val: &Dictionary) -> CallResponse<()> {
        self.call("nvim_set_hl", (ns_id, name, val)).await
    }
    async fn nvim_set_hl_ns(self, ns_id: i64) -> CallResponse<()> {
        self.call("nvim_set_hl_ns", (ns_id,)).await
    }
    async fn nvim_set_hl_ns_fast(self, ns_id: i64) -> CallResponse<()> {
        self.call("nvim_set_hl_ns_fast", (ns_id,)).await
    }
    async fn nvim_feedkeys(self, keys: &str, mode: &str, escape_ks: bool) -> CallResponse<()> {
        self.call("nvim_feedkeys", (keys, mode, escape_ks)).await
    }
    async fn nvim_input(self, keys: &str) -> CallResponse<i64> {
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
    ) -> CallResponse<()> {
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
    ) -> CallResponse<String> {
        self.call("nvim_replace_termcodes", (str, from_part, do_lt, special))
            .await
    }
    async fn nvim_exec_lua(self, code: &str, args: Vec<rmpv::Value>) -> CallResponse<Object> {
        self.call("nvim_exec_lua", (code, args)).await
    }
    async fn nvim_notify(
        self,
        msg: &str,
        log_level: i64,
        opts: &Dictionary,
    ) -> CallResponse<Object> {
        self.call("nvim_notify", (msg, log_level, opts)).await
    }
    async fn nvim_strwidth(self, text: &str) -> CallResponse<i64> {
        self.call("nvim_strwidth", (text,)).await
    }
    async fn nvim_list_runtime_paths(self) -> CallResponse<Vec<String>> {
        self.call("nvim_list_runtime_paths", ()).await
    }
    async fn nvim_get_runtime_file(self, name: &str, all: bool) -> CallResponse<Vec<String>> {
        self.call("nvim_get_runtime_file", (name, all)).await
    }
    async fn nvim_set_current_dir(self, dir: &str) -> CallResponse<()> {
        self.call("nvim_set_current_dir", (dir,)).await
    }
    async fn nvim_get_current_line(self) -> CallResponse<String> {
        self.call("nvim_get_current_line", ()).await
    }
    async fn nvim_set_current_line(self, line: &str) -> CallResponse<()> {
        self.call("nvim_set_current_line", (line,)).await
    }
    async fn nvim_del_current_line(self) -> CallResponse<()> {
        self.call("nvim_del_current_line", ()).await
    }
    async fn nvim_get_var(self, name: &str) -> CallResponse<Object> {
        self.call("nvim_get_var", (name,)).await
    }
    async fn nvim_set_var<T2: serde::Serialize>(self, name: &str, value: &T2) -> CallResponse<()> {
        self.call("nvim_set_var", (name, value)).await
    }
    async fn nvim_del_var(self, name: &str) -> CallResponse<()> {
        self.call("nvim_del_var", (name,)).await
    }
    async fn nvim_get_vvar(self, name: &str) -> CallResponse<Object> {
        self.call("nvim_get_vvar", (name,)).await
    }
    async fn nvim_set_vvar<T2: serde::Serialize>(self, name: &str, value: &T2) -> CallResponse<()> {
        self.call("nvim_set_vvar", (name, value)).await
    }
    async fn nvim_echo(
        self,
        chunks: Vec<rmpv::Value>,
        history: bool,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call("nvim_echo", (chunks, history, opts)).await
    }
    async fn nvim_out_write(self, str: &str) -> CallResponse<()> {
        self.call("nvim_out_write", (str,)).await
    }
    async fn nvim_err_write(self, str: &str) -> CallResponse<()> {
        self.call("nvim_err_write", (str,)).await
    }
    async fn nvim_err_writeln(self, str: &str) -> CallResponse<()> {
        self.call("nvim_err_writeln", (str,)).await
    }
    async fn nvim_list_bufs(self) -> CallResponse<Vec<Buffer>> {
        self.call("nvim_list_bufs", ()).await
    }
    async fn nvim_get_current_buf(self) -> CallResponse<Buffer> {
        self.call("nvim_get_current_buf", ()).await
    }
    async fn nvim_set_current_buf(self, buffer: &Buffer) -> CallResponse<()> {
        self.call("nvim_set_current_buf", (buffer,)).await
    }
    async fn nvim_list_wins(self) -> CallResponse<Vec<Window>> {
        self.call("nvim_list_wins", ()).await
    }
    async fn nvim_get_current_win(self) -> CallResponse<Window> {
        self.call("nvim_get_current_win", ()).await
    }
    async fn nvim_set_current_win(self, window: &Window) -> CallResponse<()> {
        self.call("nvim_set_current_win", (window,)).await
    }
    async fn nvim_create_buf(self, listed: bool, scratch: bool) -> CallResponse<Buffer> {
        self.call("nvim_create_buf", (listed, scratch)).await
    }
    async fn nvim_open_term(self, buffer: &Buffer, opts: &Dictionary) -> CallResponse<i64> {
        self.call("nvim_open_term", (buffer, opts)).await
    }
    async fn nvim_chan_send(self, chan: i64, data: &str) -> CallResponse<()> {
        self.call("nvim_chan_send", (chan, data)).await
    }
    async fn nvim_list_tabpages(self) -> CallResponse<Vec<Tabpage>> {
        self.call("nvim_list_tabpages", ()).await
    }
    async fn nvim_get_current_tabpage(self) -> CallResponse<Tabpage> {
        self.call("nvim_get_current_tabpage", ()).await
    }
    async fn nvim_set_current_tabpage(self, tabpage: &Tabpage) -> CallResponse<()> {
        self.call("nvim_set_current_tabpage", (tabpage,)).await
    }
    async fn nvim_paste(self, data: &str, crlf: bool, phase: i64) -> CallResponse<bool> {
        self.call("nvim_paste", (data, crlf, phase)).await
    }
    async fn nvim_put(
        self,
        lines: Vec<String>,
        _type: &str,
        after: bool,
        follow: bool,
    ) -> CallResponse<()> {
        self.call("nvim_put", (lines, _type, after, follow)).await
    }
    async fn nvim_subscribe(self, event: &str) -> CallResponse<()> {
        self.call("nvim_subscribe", (event,)).await
    }
    async fn nvim_unsubscribe(self, event: &str) -> CallResponse<()> {
        self.call("nvim_unsubscribe", (event,)).await
    }
    async fn nvim_get_color_by_name(self, name: &str) -> CallResponse<i64> {
        self.call("nvim_get_color_by_name", (name,)).await
    }
    async fn nvim_get_color_map(self) -> CallResponse<Dictionary> {
        self.call("nvim_get_color_map", ()).await
    }
    async fn nvim_get_context(self, opts: &Dictionary) -> CallResponse<Dictionary> {
        self.call("nvim_get_context", (opts,)).await
    }
    async fn nvim_load_context(self, dict: &Dictionary) -> CallResponse<Object> {
        self.call("nvim_load_context", (dict,)).await
    }
    async fn nvim_get_mode(self) -> CallResponse<Dictionary> {
        self.call("nvim_get_mode", ()).await
    }
    async fn nvim_get_keymap(self, mode: &str) -> CallResponse<Vec<Dictionary>> {
        self.call("nvim_get_keymap", (mode,)).await
    }
    async fn nvim_set_keymap(
        self,
        mode: &str,
        lhs: &str,
        rhs: &str,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call("nvim_set_keymap", (mode, lhs, rhs, opts)).await
    }
    async fn nvim_del_keymap(self, mode: &str, lhs: &str) -> CallResponse<()> {
        self.call("nvim_del_keymap", (mode, lhs)).await
    }
    async fn nvim_get_api_info(self) -> CallResponse<Vec<rmpv::Value>> {
        self.call("nvim_get_api_info", ()).await
    }
    async fn nvim_set_client_info(
        self,
        name: &str,
        version: &Dictionary,
        _type: &str,
        methods: &Dictionary,
        attributes: &Dictionary,
    ) -> CallResponse<()> {
        self.call(
            "nvim_set_client_info",
            (name, version, _type, methods, attributes),
        )
        .await
    }
    async fn nvim_get_chan_info(self, chan: i64) -> CallResponse<Dictionary> {
        self.call("nvim_get_chan_info", (chan,)).await
    }
    async fn nvim_list_chans(self) -> CallResponse<Vec<rmpv::Value>> {
        self.call("nvim_list_chans", ()).await
    }
    async fn nvim_call_atomic(self, calls: Vec<rmpv::Value>) -> CallResponse<Vec<rmpv::Value>> {
        self.call("nvim_call_atomic", (calls,)).await
    }
    async fn nvim_list_uis(self) -> CallResponse<Vec<rmpv::Value>> {
        self.call("nvim_list_uis", ()).await
    }
    async fn nvim_get_proc_children(self, pid: i64) -> CallResponse<Vec<rmpv::Value>> {
        self.call("nvim_get_proc_children", (pid,)).await
    }
    async fn nvim_get_proc(self, pid: i64) -> CallResponse<Object> {
        self.call("nvim_get_proc", (pid,)).await
    }
    async fn nvim_select_popupmenu_item(
        self,
        item: i64,
        insert: bool,
        finish: bool,
        opts: &Dictionary,
    ) -> CallResponse<()> {
        self.call("nvim_select_popupmenu_item", (item, insert, finish, opts))
            .await
    }
    async fn nvim_del_mark(self, name: &str) -> CallResponse<bool> {
        self.call("nvim_del_mark", (name,)).await
    }
    async fn nvim_get_mark(self, name: &str, opts: &Dictionary) -> CallResponse<Vec<rmpv::Value>> {
        self.call("nvim_get_mark", (name, opts)).await
    }
    async fn nvim_eval_statusline(self, str: &str, opts: &Dictionary) -> CallResponse<Dictionary> {
        self.call("nvim_eval_statusline", (str, opts)).await
    }
    async fn nvim_exec2(self, src: &str, opts: &Dictionary) -> CallResponse<Dictionary> {
        self.call("nvim_exec2", (src, opts)).await
    }
    async fn nvim_command(self, command: &str) -> CallResponse<()> {
        self.call("nvim_command", (command,)).await
    }
    async fn nvim_eval(self, expr: &str) -> CallResponse<Object> {
        self.call("nvim_eval", (expr,)).await
    }
    async fn nvim_call_function(self, _fn: &str, args: Vec<rmpv::Value>) -> CallResponse<Object> {
        self.call("nvim_call_function", (_fn, args)).await
    }
    async fn nvim_call_dict_function<T1: serde::Serialize>(
        self,
        dict: &T1,
        _fn: &str,
        args: Vec<rmpv::Value>,
    ) -> CallResponse<Object> {
        self.call("nvim_call_dict_function", (dict, _fn, args))
            .await
    }
    async fn nvim_parse_expression(
        self,
        expr: &str,
        flags: &str,
        highlight: bool,
    ) -> CallResponse<Dictionary> {
        self.call("nvim_parse_expression", (expr, flags, highlight))
            .await
    }
    async fn nvim_open_win(
        self,
        buffer: &Buffer,
        enter: bool,
        config: &Dictionary,
    ) -> CallResponse<Window> {
        self.call("nvim_open_win", (buffer, enter, config)).await
    }
    async fn nvim_win_set_config(self, window: &Window, config: &Dictionary) -> CallResponse<()> {
        self.call("nvim_win_set_config", (window, config)).await
    }
    async fn nvim_win_get_config(self, window: &Window) -> CallResponse<Dictionary> {
        self.call("nvim_win_get_config", (window,)).await
    }
    async fn nvim_win_get_buf(self, window: &Window) -> CallResponse<Buffer> {
        self.call("nvim_win_get_buf", (window,)).await
    }
    async fn nvim_win_set_buf(self, window: &Window, buffer: &Buffer) -> CallResponse<()> {
        self.call("nvim_win_set_buf", (window, buffer)).await
    }
    async fn nvim_win_get_cursor(self, window: &Window) -> CallResponse<(i64, i64)> {
        self.call("nvim_win_get_cursor", (window,)).await
    }
    async fn nvim_win_set_cursor(self, window: &Window, pos: (i64, i64)) -> CallResponse<()> {
        self.call("nvim_win_set_cursor", (window, pos)).await
    }
    async fn nvim_win_get_height(self, window: &Window) -> CallResponse<i64> {
        self.call("nvim_win_get_height", (window,)).await
    }
    async fn nvim_win_set_height(self, window: &Window, height: i64) -> CallResponse<()> {
        self.call("nvim_win_set_height", (window, height)).await
    }
    async fn nvim_win_get_width(self, window: &Window) -> CallResponse<i64> {
        self.call("nvim_win_get_width", (window,)).await
    }
    async fn nvim_win_set_width(self, window: &Window, width: i64) -> CallResponse<()> {
        self.call("nvim_win_set_width", (window, width)).await
    }
    async fn nvim_win_get_var(self, window: &Window, name: &str) -> CallResponse<Object> {
        self.call("nvim_win_get_var", (window, name)).await
    }
    async fn nvim_win_set_var<T3: serde::Serialize>(
        self,
        window: &Window,
        name: &str,
        value: &T3,
    ) -> CallResponse<()> {
        self.call("nvim_win_set_var", (window, name, value)).await
    }
    async fn nvim_win_del_var(self, window: &Window, name: &str) -> CallResponse<()> {
        self.call("nvim_win_del_var", (window, name)).await
    }
    async fn nvim_win_get_position(self, window: &Window) -> CallResponse<(i64, i64)> {
        self.call("nvim_win_get_position", (window,)).await
    }
    async fn nvim_win_get_tabpage(self, window: &Window) -> CallResponse<Tabpage> {
        self.call("nvim_win_get_tabpage", (window,)).await
    }
    async fn nvim_win_get_number(self, window: &Window) -> CallResponse<i64> {
        self.call("nvim_win_get_number", (window,)).await
    }
    async fn nvim_win_is_valid(self, window: &Window) -> CallResponse<bool> {
        self.call("nvim_win_is_valid", (window,)).await
    }
    async fn nvim_win_hide(self, window: &Window) -> CallResponse<()> {
        self.call("nvim_win_hide", (window,)).await
    }
    async fn nvim_win_close(self, window: &Window, force: bool) -> CallResponse<()> {
        self.call("nvim_win_close", (window, force)).await
    }
    async fn nvim_win_call(self, window: &Window, fun: &LuaRef) -> CallResponse<Object> {
        self.call("nvim_win_call", (window, fun)).await
    }
    async fn nvim_win_set_hl_ns(self, window: &Window, ns_id: i64) -> CallResponse<()> {
        self.call("nvim_win_set_hl_ns", (window, ns_id)).await
    }
}
