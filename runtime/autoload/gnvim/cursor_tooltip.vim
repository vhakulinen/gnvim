command! -complete=customlist,s:complete
            \ -nargs=1 CursorTooltipStyle call s:set_style(<q-args>)

function! s:set_style(style)
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'CursorTooltipSetStyle', a:style)
endfunction

function! s:get_styles()
    return gnvim#cursor_tooltip#get_styles()
endfunction

function! s:complete(lead, line, pos)

    let items = []

    for item in s:get_styles()
        if match(item, a:lead) == 0
            let items += [ item ]
        endif
    endfor

    return items
endfunction

function! gnvim#cursor_tooltip#get_styles()
    return rpcrequest(g:gnvim_channel_id, 'Gnvim', 'CursorTooltipGetStyles')
endfunction

function! gnvim#cursor_tooltip#show(content, row, col)
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'CursorTooltipShow', a:content, a:row, a:col)
endfunction

function! gnvim#cursor_tooltip#hide()
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'CursorTooltipHide')
endfunction
