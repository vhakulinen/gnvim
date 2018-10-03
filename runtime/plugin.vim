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
                \}

    call rpcnotify(0, 'Gnvim', 'SetGuiColors', colors)
endfunction

augroup GnvimColors
    autocmd!
    autocmd ColorScheme * call SetGuiColors()
augroup END

call SetGuiColors()
