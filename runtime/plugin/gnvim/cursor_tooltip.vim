command! -complete=customlist,s:complete
            \ -nargs=1 CursorTooltipStyle call s:set_style(<q-args>)

function! s:set_style(style)
    call rpcnotify(0, 'Gnvim', 'SetCursorTooltipStyle', a:style)
endfunction

function! s:get_styles()
    return gnvim#cursor_tooltip#get_styles()
endfunction

function! s:complete(lead, line, pos)

    let items = []

    for item in s:get_styles()
        "if item =~ a:lead
        if match(item, a:lead) == 0
            let items += [ item ]
        endif
    endfor

    return items
endfunction

function! gnvim#cursor_tooltip#get_styles()
    return rpcrequest(1, 'Gnvim', 'GetCursorTooltipStyles')
endfunction

CursorTooltipStyle nord
