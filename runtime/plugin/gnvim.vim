if has("g:gnvim_runtime_loaded")
    finish
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

    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'SetGuiColors', colors)
endfunction

augroup GnvimColors
    autocmd!
    autocmd ColorScheme * call gnvim#set_gui_colors()
    autocmd VimEnter * call gnvim#set_gui_colors()
augroup END

inoremap <expr> <C-s> gnvim#popupmenu#toggle_details()
