use crate::rpc::{RpcWriter, WriteError};
use crate::{
    args,
    types::{Buffer, Dictionary, LuaRef, Object, Tabpage, UiOptions, Window},
    CallResponse, Client,
};
impl<W: RpcWriter> Client<W> {
    pub async fn nvim_get_autocmds(
        &mut self,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_autocmds", args![opts]).await
    }
    pub async fn nvim_create_autocmd(
        &mut self,
        event: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_create_autocmd", args![event, opts]).await
    }
    pub async fn nvim_del_autocmd(&mut self, id: i64) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_autocmd", args![id]).await
    }
    pub async fn nvim_clear_autocmds(
        &mut self,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_clear_autocmds", args![opts]).await
    }
    pub async fn nvim_create_augroup(
        &mut self,
        name: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_create_augroup", args![name, opts]).await
    }
    pub async fn nvim_del_augroup_by_id(
        &mut self,
        id: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_augroup_by_id", args![id]).await
    }
    pub async fn nvim_del_augroup_by_name(
        &mut self,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_augroup_by_name", args![name]).await
    }
    pub async fn nvim_exec_autocmds(
        &mut self,
        event: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_exec_autocmds", args![event, opts]).await
    }
    pub async fn nvim_buf_line_count(
        &mut self,
        buffer: &Buffer,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_buf_line_count", args![buffer]).await
    }
    pub async fn nvim_buf_attach(
        &mut self,
        buffer: &Buffer,
        send_buffer: bool,
        opts: &Dictionary,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_attach", args![buffer, send_buffer, opts])
            .await
    }
    pub async fn nvim_buf_detach(
        &mut self,
        buffer: &Buffer,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_detach", args![buffer]).await
    }
    pub async fn nvim_buf_get_lines(
        &mut self,
        buffer: &Buffer,
        start: i64,
        end: i64,
        strict_indexing: bool,
    ) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call(
            "nvim_buf_get_lines",
            args![buffer, start, end, strict_indexing],
        )
        .await
    }
    pub async fn nvim_buf_set_lines(
        &mut self,
        buffer: &Buffer,
        start: i64,
        end: i64,
        strict_indexing: bool,
        replacement: Vec<String>,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_set_lines",
            args![buffer, start, end, strict_indexing, replacement],
        )
        .await
    }
    pub async fn nvim_buf_set_text(
        &mut self,
        buffer: &Buffer,
        start_row: i64,
        start_col: i64,
        end_row: i64,
        end_col: i64,
        replacement: Vec<String>,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_set_text",
            args![buffer, start_row, start_col, end_row, end_col, replacement],
        )
        .await
    }
    pub async fn nvim_buf_get_text(
        &mut self,
        buffer: &Buffer,
        start_row: i64,
        start_col: i64,
        end_row: i64,
        end_col: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call(
            "nvim_buf_get_text",
            args![buffer, start_row, start_col, end_row, end_col, opts],
        )
        .await
    }
    pub async fn nvim_buf_get_offset(
        &mut self,
        buffer: &Buffer,
        index: i64,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_buf_get_offset", args![buffer, index]).await
    }
    pub async fn nvim_buf_get_var(
        &mut self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_buf_get_var", args![buffer, name]).await
    }
    pub async fn nvim_buf_get_changedtick(
        &mut self,
        buffer: &Buffer,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_buf_get_changedtick", args![buffer]).await
    }
    pub async fn nvim_buf_get_keymap(
        &mut self,
        buffer: &Buffer,
        mode: &str,
    ) -> Result<CallResponse<Vec<Dictionary>>, WriteError> {
        self.call("nvim_buf_get_keymap", args![buffer, mode]).await
    }
    pub async fn nvim_buf_set_keymap(
        &mut self,
        buffer: &Buffer,
        mode: &str,
        lhs: &str,
        rhs: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_keymap", args![buffer, mode, lhs, rhs, opts])
            .await
    }
    pub async fn nvim_buf_del_keymap(
        &mut self,
        buffer: &Buffer,
        mode: &str,
        lhs: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_del_keymap", args![buffer, mode, lhs])
            .await
    }
    pub async fn nvim_buf_set_var(
        &mut self,
        buffer: &Buffer,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_var", args![buffer, name, value])
            .await
    }
    pub async fn nvim_buf_del_var(
        &mut self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_del_var", args![buffer, name]).await
    }
    pub async fn nvim_buf_get_name(
        &mut self,
        buffer: &Buffer,
    ) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_buf_get_name", args![buffer]).await
    }
    pub async fn nvim_buf_set_name(
        &mut self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_name", args![buffer, name]).await
    }
    pub async fn nvim_buf_is_loaded(
        &mut self,
        buffer: &Buffer,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_is_loaded", args![buffer]).await
    }
    pub async fn nvim_buf_delete(
        &mut self,
        buffer: &Buffer,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_delete", args![buffer, opts]).await
    }
    pub async fn nvim_buf_is_valid(
        &mut self,
        buffer: &Buffer,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_is_valid", args![buffer]).await
    }
    pub async fn nvim_buf_del_mark(
        &mut self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_del_mark", args![buffer, name]).await
    }
    pub async fn nvim_buf_set_mark(
        &mut self,
        buffer: &Buffer,
        name: &str,
        line: i64,
        col: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_set_mark", args![buffer, name, line, col, opts])
            .await
    }
    pub async fn nvim_buf_get_mark(
        &mut self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<(i64, i64)>, WriteError> {
        self.call("nvim_buf_get_mark", args![buffer, name]).await
    }
    pub async fn nvim_buf_call(
        &mut self,
        buffer: &Buffer,
        fun: &LuaRef,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_buf_call", args![buffer, fun]).await
    }
    pub async fn nvim_parse_cmd(
        &mut self,
        str: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_parse_cmd", args![str, opts]).await
    }
    pub async fn nvim_cmd(
        &mut self,
        cmd: &Dictionary,
        opts: &Dictionary,
    ) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_cmd", args![cmd, opts]).await
    }
    pub async fn nvim_create_user_command(
        &mut self,
        name: &str,
        command: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_create_user_command", args![name, command, opts])
            .await
    }
    pub async fn nvim_del_user_command(
        &mut self,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_user_command", args![name]).await
    }
    pub async fn nvim_buf_create_user_command(
        &mut self,
        buffer: &Buffer,
        name: &str,
        command: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_create_user_command",
            args![buffer, name, command, opts],
        )
        .await
    }
    pub async fn nvim_buf_del_user_command(
        &mut self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_del_user_command", args![buffer, name])
            .await
    }
    pub async fn nvim_get_commands(
        &mut self,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_commands", args![opts]).await
    }
    pub async fn nvim_buf_get_commands(
        &mut self,
        buffer: &Buffer,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_buf_get_commands", args![buffer, opts])
            .await
    }
    pub async fn nvim_create_namespace(
        &mut self,
        name: &str,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_create_namespace", args![name]).await
    }
    pub async fn nvim_get_namespaces(&mut self) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_namespaces", args![]).await
    }
    pub async fn nvim_buf_get_extmark_by_id(
        &mut self,
        buffer: &Buffer,
        ns_id: i64,
        id: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<i64>>, WriteError> {
        self.call("nvim_buf_get_extmark_by_id", args![buffer, ns_id, id, opts])
            .await
    }
    pub async fn nvim_buf_get_extmarks(
        &mut self,
        buffer: &Buffer,
        ns_id: i64,
        start: &Object,
        end: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call(
            "nvim_buf_get_extmarks",
            args![buffer, ns_id, start, end, opts],
        )
        .await
    }
    pub async fn nvim_buf_set_extmark(
        &mut self,
        buffer: &Buffer,
        ns_id: i64,
        line: i64,
        col: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call(
            "nvim_buf_set_extmark",
            args![buffer, ns_id, line, col, opts],
        )
        .await
    }
    pub async fn nvim_buf_del_extmark(
        &mut self,
        buffer: &Buffer,
        ns_id: i64,
        id: i64,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_buf_del_extmark", args![buffer, ns_id, id])
            .await
    }
    pub async fn nvim_buf_add_highlight(
        &mut self,
        buffer: &Buffer,
        ns_id: i64,
        hl_group: &str,
        line: i64,
        col_start: i64,
        col_end: i64,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call(
            "nvim_buf_add_highlight",
            args![buffer, ns_id, hl_group, line, col_start, col_end],
        )
        .await
    }
    pub async fn nvim_buf_clear_namespace(
        &mut self,
        buffer: &Buffer,
        ns_id: i64,
        line_start: i64,
        line_end: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_buf_clear_namespace",
            args![buffer, ns_id, line_start, line_end],
        )
        .await
    }
    pub async fn nvim_set_decoration_provider(
        &mut self,
        ns_id: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_decoration_provider", args![ns_id, opts])
            .await
    }
    pub async fn nvim_get_option_value(
        &mut self,
        name: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_option_value", args![name, opts]).await
    }
    pub async fn nvim_set_option_value(
        &mut self,
        name: &str,
        value: &Object,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_option_value", args![name, value, opts])
            .await
    }
    pub async fn nvim_get_all_options_info(
        &mut self,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_all_options_info", args![]).await
    }
    pub async fn nvim_get_option_info(
        &mut self,
        name: &str,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_option_info", args![name]).await
    }
    pub async fn nvim_set_option(
        &mut self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_option", args![name, value]).await
    }
    pub async fn nvim_get_option(
        &mut self,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_option", args![name]).await
    }
    pub async fn nvim_buf_get_option(
        &mut self,
        buffer: &Buffer,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_buf_get_option", args![buffer, name]).await
    }
    pub async fn nvim_buf_set_option(
        &mut self,
        buffer: &Buffer,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_buf_set_option", args![buffer, name, value])
            .await
    }
    pub async fn nvim_win_get_option(
        &mut self,
        window: &Window,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_win_get_option", args![window, name]).await
    }
    pub async fn nvim_win_set_option(
        &mut self,
        window: &Window,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_option", args![window, name, value])
            .await
    }
    pub async fn nvim_tabpage_list_wins(
        &mut self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<Vec<Window>>, WriteError> {
        self.call("nvim_tabpage_list_wins", args![tabpage]).await
    }
    pub async fn nvim_tabpage_get_var(
        &mut self,
        tabpage: &Tabpage,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_tabpage_get_var", args![tabpage, name])
            .await
    }
    pub async fn nvim_tabpage_set_var(
        &mut self,
        tabpage: &Tabpage,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_tabpage_set_var", args![tabpage, name, value])
            .await
    }
    pub async fn nvim_tabpage_del_var(
        &mut self,
        tabpage: &Tabpage,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_tabpage_del_var", args![tabpage, name])
            .await
    }
    pub async fn nvim_tabpage_get_win(
        &mut self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<Window>, WriteError> {
        self.call("nvim_tabpage_get_win", args![tabpage]).await
    }
    pub async fn nvim_tabpage_get_number(
        &mut self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_tabpage_get_number", args![tabpage]).await
    }
    pub async fn nvim_tabpage_is_valid(
        &mut self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_tabpage_is_valid", args![tabpage]).await
    }
    pub async fn nvim_ui_attach(
        &mut self,
        width: i64,
        height: i64,
        options: UiOptions,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_attach", args![width, height, options])
            .await
    }
    pub async fn nvim_ui_detach(&mut self) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_detach", args![]).await
    }
    pub async fn nvim_ui_try_resize(
        &mut self,
        width: i64,
        height: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_try_resize", args![width, height]).await
    }
    pub async fn nvim_ui_set_option(
        &mut self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_set_option", args![name, value]).await
    }
    pub async fn nvim_ui_try_resize_grid(
        &mut self,
        grid: i64,
        width: i64,
        height: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_try_resize_grid", args![grid, width, height])
            .await
    }
    pub async fn nvim_ui_pum_set_height(
        &mut self,
        height: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_pum_set_height", args![height]).await
    }
    pub async fn nvim_ui_pum_set_bounds(
        &mut self,
        width: f64,
        height: f64,
        row: f64,
        col: f64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_ui_pum_set_bounds", args![width, height, row, col])
            .await
    }
    pub async fn nvim_get_hl_by_name(
        &mut self,
        name: &str,
        rgb: bool,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_hl_by_name", args![name, rgb]).await
    }
    pub async fn nvim_get_hl_by_id(
        &mut self,
        hl_id: i64,
        rgb: bool,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_hl_by_id", args![hl_id, rgb]).await
    }
    pub async fn nvim_get_hl_id_by_name(
        &mut self,
        name: &str,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_get_hl_id_by_name", args![name]).await
    }
    pub async fn nvim_set_hl(
        &mut self,
        ns_id: i64,
        name: &str,
        val: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_hl", args![ns_id, name, val]).await
    }
    pub async fn nvim_set_hl_ns(&mut self, ns_id: i64) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_hl_ns", args![ns_id]).await
    }
    pub async fn nvim_set_hl_ns_fast(
        &mut self,
        ns_id: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_hl_ns_fast", args![ns_id]).await
    }
    pub async fn nvim_feedkeys(
        &mut self,
        keys: &str,
        mode: &str,
        escape_ks: bool,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_feedkeys", args![keys, mode, escape_ks])
            .await
    }
    pub async fn nvim_input(&mut self, keys: &str) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_input", args![keys]).await
    }
    pub async fn nvim_input_mouse(
        &mut self,
        button: &str,
        action: &str,
        modifier: &str,
        grid: i64,
        row: i64,
        col: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_input_mouse",
            args![button, action, modifier, grid, row, col],
        )
        .await
    }
    pub async fn nvim_replace_termcodes(
        &mut self,
        str: &str,
        from_part: bool,
        do_lt: bool,
        special: bool,
    ) -> Result<CallResponse<String>, WriteError> {
        self.call(
            "nvim_replace_termcodes",
            args![str, from_part, do_lt, special],
        )
        .await
    }
    pub async fn nvim_exec_lua(
        &mut self,
        code: &str,
        args: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_exec_lua", args![code, args]).await
    }
    pub async fn nvim_notify(
        &mut self,
        msg: &str,
        log_level: i64,
        opts: &Dictionary,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_notify", args![msg, log_level, opts]).await
    }
    pub async fn nvim_strwidth(&mut self, text: &str) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_strwidth", args![text]).await
    }
    pub async fn nvim_list_runtime_paths(
        &mut self,
    ) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call("nvim_list_runtime_paths", args![]).await
    }
    pub async fn nvim_get_runtime_file(
        &mut self,
        name: &str,
        all: bool,
    ) -> Result<CallResponse<Vec<String>>, WriteError> {
        self.call("nvim_get_runtime_file", args![name, all]).await
    }
    pub async fn nvim_set_current_dir(
        &mut self,
        dir: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_dir", args![dir]).await
    }
    pub async fn nvim_get_current_line(&mut self) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_get_current_line", args![]).await
    }
    pub async fn nvim_set_current_line(
        &mut self,
        line: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_line", args![line]).await
    }
    pub async fn nvim_del_current_line(&mut self) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_current_line", args![]).await
    }
    pub async fn nvim_get_var(&mut self, name: &str) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_var", args![name]).await
    }
    pub async fn nvim_set_var(
        &mut self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_var", args![name, value]).await
    }
    pub async fn nvim_del_var(&mut self, name: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_var", args![name]).await
    }
    pub async fn nvim_get_vvar(&mut self, name: &str) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_vvar", args![name]).await
    }
    pub async fn nvim_set_vvar(
        &mut self,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_vvar", args![name, value]).await
    }
    pub async fn nvim_echo(
        &mut self,
        chunks: Vec<rmpv::Value>,
        history: bool,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_echo", args![chunks, history, opts]).await
    }
    pub async fn nvim_out_write(&mut self, str: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_out_write", args![str]).await
    }
    pub async fn nvim_err_write(&mut self, str: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_err_write", args![str]).await
    }
    pub async fn nvim_err_writeln(&mut self, str: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_err_writeln", args![str]).await
    }
    pub async fn nvim_list_bufs(&mut self) -> Result<CallResponse<Vec<Buffer>>, WriteError> {
        self.call("nvim_list_bufs", args![]).await
    }
    pub async fn nvim_get_current_buf(&mut self) -> Result<CallResponse<Buffer>, WriteError> {
        self.call("nvim_get_current_buf", args![]).await
    }
    pub async fn nvim_set_current_buf(
        &mut self,
        buffer: &Buffer,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_buf", args![buffer]).await
    }
    pub async fn nvim_list_wins(&mut self) -> Result<CallResponse<Vec<Window>>, WriteError> {
        self.call("nvim_list_wins", args![]).await
    }
    pub async fn nvim_get_current_win(&mut self) -> Result<CallResponse<Window>, WriteError> {
        self.call("nvim_get_current_win", args![]).await
    }
    pub async fn nvim_set_current_win(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_win", args![window]).await
    }
    pub async fn nvim_create_buf(
        &mut self,
        listed: bool,
        scratch: bool,
    ) -> Result<CallResponse<Buffer>, WriteError> {
        self.call("nvim_create_buf", args![listed, scratch]).await
    }
    pub async fn nvim_open_term(
        &mut self,
        buffer: &Buffer,
        opts: &Dictionary,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_open_term", args![buffer, opts]).await
    }
    pub async fn nvim_chan_send(
        &mut self,
        chan: i64,
        data: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_chan_send", args![chan, data]).await
    }
    pub async fn nvim_list_tabpages(&mut self) -> Result<CallResponse<Vec<Tabpage>>, WriteError> {
        self.call("nvim_list_tabpages", args![]).await
    }
    pub async fn nvim_get_current_tabpage(&mut self) -> Result<CallResponse<Tabpage>, WriteError> {
        self.call("nvim_get_current_tabpage", args![]).await
    }
    pub async fn nvim_set_current_tabpage(
        &mut self,
        tabpage: &Tabpage,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_current_tabpage", args![tabpage]).await
    }
    pub async fn nvim_paste(
        &mut self,
        data: &str,
        crlf: bool,
        phase: i64,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_paste", args![data, crlf, phase]).await
    }
    pub async fn nvim_put(
        &mut self,
        lines: Vec<String>,
        _type: &str,
        after: bool,
        follow: bool,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_put", args![lines, _type, after, follow])
            .await
    }
    pub async fn nvim_subscribe(&mut self, event: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_subscribe", args![event]).await
    }
    pub async fn nvim_unsubscribe(&mut self, event: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_unsubscribe", args![event]).await
    }
    pub async fn nvim_get_color_by_name(
        &mut self,
        name: &str,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_get_color_by_name", args![name]).await
    }
    pub async fn nvim_get_color_map(&mut self) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_color_map", args![]).await
    }
    pub async fn nvim_get_context(
        &mut self,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_context", args![opts]).await
    }
    pub async fn nvim_load_context(
        &mut self,
        dict: &Dictionary,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_load_context", args![dict]).await
    }
    pub async fn nvim_get_mode(&mut self) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_mode", args![]).await
    }
    pub async fn nvim_get_keymap(
        &mut self,
        mode: &str,
    ) -> Result<CallResponse<Vec<Dictionary>>, WriteError> {
        self.call("nvim_get_keymap", args![mode]).await
    }
    pub async fn nvim_set_keymap(
        &mut self,
        mode: &str,
        lhs: &str,
        rhs: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_set_keymap", args![mode, lhs, rhs, opts])
            .await
    }
    pub async fn nvim_del_keymap(
        &mut self,
        mode: &str,
        lhs: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_del_keymap", args![mode, lhs]).await
    }
    pub async fn nvim_get_api_info(
        &mut self,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_api_info", args![]).await
    }
    pub async fn nvim_set_client_info(
        &mut self,
        name: &str,
        version: &Dictionary,
        _type: &str,
        methods: &Dictionary,
        attributes: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_set_client_info",
            args![name, version, _type, methods, attributes],
        )
        .await
    }
    pub async fn nvim_get_chan_info(
        &mut self,
        chan: i64,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_get_chan_info", args![chan]).await
    }
    pub async fn nvim_list_chans(&mut self) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_list_chans", args![]).await
    }
    pub async fn nvim_call_atomic(
        &mut self,
        calls: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_call_atomic", args![calls]).await
    }
    pub async fn nvim_list_uis(&mut self) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_list_uis", args![]).await
    }
    pub async fn nvim_get_proc_children(
        &mut self,
        pid: i64,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_proc_children", args![pid]).await
    }
    pub async fn nvim_get_proc(&mut self, pid: i64) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_get_proc", args![pid]).await
    }
    pub async fn nvim_select_popupmenu_item(
        &mut self,
        item: i64,
        insert: bool,
        finish: bool,
        opts: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call(
            "nvim_select_popupmenu_item",
            args![item, insert, finish, opts],
        )
        .await
    }
    pub async fn nvim_del_mark(&mut self, name: &str) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_del_mark", args![name]).await
    }
    pub async fn nvim_get_mark(
        &mut self,
        name: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Vec<rmpv::Value>>, WriteError> {
        self.call("nvim_get_mark", args![name, opts]).await
    }
    pub async fn nvim_eval_statusline(
        &mut self,
        str: &str,
        opts: &Dictionary,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_eval_statusline", args![str, opts]).await
    }
    pub async fn nvim_exec(
        &mut self,
        src: &str,
        output: bool,
    ) -> Result<CallResponse<String>, WriteError> {
        self.call("nvim_exec", args![src, output]).await
    }
    pub async fn nvim_command(&mut self, command: &str) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_command", args![command]).await
    }
    pub async fn nvim_eval(&mut self, expr: &str) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_eval", args![expr]).await
    }
    pub async fn nvim_call_function(
        &mut self,
        _fn: &str,
        args: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_call_function", args![_fn, args]).await
    }
    pub async fn nvim_call_dict_function(
        &mut self,
        dict: &Object,
        _fn: &str,
        args: Vec<rmpv::Value>,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_call_dict_function", args![dict, _fn, args])
            .await
    }
    pub async fn nvim_parse_expression(
        &mut self,
        expr: &str,
        flags: &str,
        highlight: bool,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_parse_expression", args![expr, flags, highlight])
            .await
    }
    pub async fn nvim_open_win(
        &mut self,
        buffer: &Buffer,
        enter: bool,
        config: &Dictionary,
    ) -> Result<CallResponse<Window>, WriteError> {
        self.call("nvim_open_win", args![buffer, enter, config])
            .await
    }
    pub async fn nvim_win_set_config(
        &mut self,
        window: &Window,
        config: &Dictionary,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_config", args![window, config])
            .await
    }
    pub async fn nvim_win_get_config(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<Dictionary>, WriteError> {
        self.call("nvim_win_get_config", args![window]).await
    }
    pub async fn nvim_win_get_buf(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<Buffer>, WriteError> {
        self.call("nvim_win_get_buf", args![window]).await
    }
    pub async fn nvim_win_set_buf(
        &mut self,
        window: &Window,
        buffer: &Buffer,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_buf", args![window, buffer]).await
    }
    pub async fn nvim_win_get_cursor(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<(i64, i64)>, WriteError> {
        self.call("nvim_win_get_cursor", args![window]).await
    }
    pub async fn nvim_win_set_cursor(
        &mut self,
        window: &Window,
        pos: (i64, i64),
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_cursor", args![window, pos]).await
    }
    pub async fn nvim_win_get_height(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_win_get_height", args![window]).await
    }
    pub async fn nvim_win_set_height(
        &mut self,
        window: &Window,
        height: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_height", args![window, height])
            .await
    }
    pub async fn nvim_win_get_width(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_win_get_width", args![window]).await
    }
    pub async fn nvim_win_set_width(
        &mut self,
        window: &Window,
        width: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_width", args![window, width]).await
    }
    pub async fn nvim_win_get_var(
        &mut self,
        window: &Window,
        name: &str,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_win_get_var", args![window, name]).await
    }
    pub async fn nvim_win_set_var(
        &mut self,
        window: &Window,
        name: &str,
        value: &Object,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_var", args![window, name, value])
            .await
    }
    pub async fn nvim_win_del_var(
        &mut self,
        window: &Window,
        name: &str,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_del_var", args![window, name]).await
    }
    pub async fn nvim_win_get_position(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<(i64, i64)>, WriteError> {
        self.call("nvim_win_get_position", args![window]).await
    }
    pub async fn nvim_win_get_tabpage(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<Tabpage>, WriteError> {
        self.call("nvim_win_get_tabpage", args![window]).await
    }
    pub async fn nvim_win_get_number(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<i64>, WriteError> {
        self.call("nvim_win_get_number", args![window]).await
    }
    pub async fn nvim_win_is_valid(
        &mut self,
        window: &Window,
    ) -> Result<CallResponse<bool>, WriteError> {
        self.call("nvim_win_is_valid", args![window]).await
    }
    pub async fn nvim_win_hide(&mut self, window: &Window) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_hide", args![window]).await
    }
    pub async fn nvim_win_close(
        &mut self,
        window: &Window,
        force: bool,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_close", args![window, force]).await
    }
    pub async fn nvim_win_call(
        &mut self,
        window: &Window,
        fun: &LuaRef,
    ) -> Result<CallResponse<Object>, WriteError> {
        self.call("nvim_win_call", args![window, fun]).await
    }
    pub async fn nvim_win_set_hl_ns(
        &mut self,
        window: &Window,
        ns_id: i64,
    ) -> Result<CallResponse<()>, WriteError> {
        self.call("nvim_win_set_hl_ns", args![window, ns_id]).await
    }
}
