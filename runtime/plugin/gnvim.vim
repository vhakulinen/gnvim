
if has("g:gnvim_runtime_loaded")
    exit
endif

let g:gnvim_runtime_loaded = 1

function! gnvim#get_hl_term(group, term)
    " Store output of group to variable
    let output = execute('hi ' . a:group)

    " Find the term we're looking for
    return matchstr(output, a:term.'=\zs\S*')
endfunction

function! gnvim#set_gui_colors()
    let colors = {
                \ 'pmenu_bg': gnvim#get_hl_term('Pmenu', 'guibg'),
                \ 'pmenu_fg': gnvim#get_hl_term('Pmenu', 'guifg'),
                \ 'pmenusel_bg': gnvim#get_hl_term('PmenuSel', 'guibg'),
                \ 'pmenusel_fg': gnvim#get_hl_term('PmenuSel', 'guifg'),
                \
                \ 'tabline_fg': gnvim#get_hl_term('TabLine', 'guifg'),
                \ 'tabline_bg': gnvim#get_hl_term('TabLine', 'guibg'),
                \ 'tablinesel_fg': gnvim#get_hl_term('TabLineSel', 'guifg'),
                \ 'tablinesel_bg': gnvim#get_hl_term('TabLineSel', 'guibg'),
                \ 'tablinefill_fg': gnvim#get_hl_term('TabLineFill', 'guifg'),
                \ 'tablinefill_bg': gnvim#get_hl_term('TabLineFill', 'guibg'),
                \
                \ 'cmdline_fg': gnvim#get_hl_term('Normal', 'guifg'),
                \ 'cmdline_bg': gnvim#get_hl_term('Normal', 'guibg'),
                \ 'cmdline_border': gnvim#get_hl_term('TabLineSel', 'guibg'),
                \
                \ 'wildmenu_bg': gnvim#get_hl_term('Pmenu', 'guibg'),
                \ 'wildmenu_fg': gnvim#get_hl_term('Pmenu', 'guifg'),
                \ 'wildmenusel_bg': gnvim#get_hl_term('PmenuSel', 'guibg'),
                \ 'wildmenusel_fg': gnvim#get_hl_term('PmenuSel', 'guifg'),
                \}

    call rpcnotify(0, 'Gnvim', 'SetGuiColors', colors)
endfunction

function! gnvim#completion_menu_toggle_info()
    call rpcnotify(0, 'Gnvim', 'CompletionMenuToggleInfo')
    return ''
endfunction

augroup GnvimColors
    autocmd!
    autocmd ColorScheme * call gnvim#set_gui_colors()
    autocmd VimEnter * call gnvim#set_gui_colors()
augroup END

augroup GnvimCursor
	autocmd!
	autocmd CursorMoved,CursorMovedI * call gnvim#hover#cursor_moved()
	"autocmd InsertEnter * call gnvim#hover#hide_hover()
augroup END

inoremap <expr> <C-s> gnvim#completion_menu_toggle_info()
