
function! ReturnHighlightTerm(group, term)
    " Store output of group to variable
    let output = execute('hi ' . a:group)

    " Find the term we're looking for
    return matchstr(output, a:term.'=\zs\S*')
endfunction

function! SetGuiColors()
    let colors = {
                \ 'pmenu_bg': ReturnHighlightTerm('Pmenu', 'guibg'),
                \ 'pmenu_fg': ReturnHighlightTerm('Pmenu', 'guifg'),
                \ 'pmenusel_bg': ReturnHighlightTerm('PmenuSel', 'guibg'),
                \ 'pmenusel_fg': ReturnHighlightTerm('PmenuSel', 'guifg'),
                \ 'tabline_fg': ReturnHighlightTerm('TabLine', 'guifg'),
                \ 'tabline_bg': ReturnHighlightTerm('TabLine', 'guibg'),
                \ 'tablinesel_fg': ReturnHighlightTerm('TabLineSel', 'guifg'),
                \ 'tablinesel_bg': ReturnHighlightTerm('TabLineSel', 'guibg'),
                \ 'tablinefill_fg': ReturnHighlightTerm('TabLineFill', 'guifg'),
                \ 'tablinefill_bg': ReturnHighlightTerm('TabLineFill', 'guibg'),
                \}

    call rpcnotify(0, 'Gnvim', 'SetGuiColors', colors)
endfunction

function! CompletionMenuToggleInfo()
    call rpcnotify(0, 'Gnvim', 'CompletionMenuToggleInfo')
    return ''
endfunction

augroup GnvimColors
    autocmd!
    autocmd ColorScheme * call SetGuiColors()
augroup END

inoremap <expr> <C-s> CompletionMenuToggleInfo()
