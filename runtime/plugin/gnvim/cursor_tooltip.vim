command! -complete=customlist,s:complete
            \ -nargs=1 CursorTooltipStyle call s:set_styles(<q-args>)

function! s:set_styles(style)
    call rpcnotify(0, 'Gnvim', 'SetCursorTooltipStyle', a:style)
endfunction

function! s:get_styles()
    let paths = split(globpath('./runtime/web-resources/styles', '*.css'), '\n')
    let styles = []

    for item in paths
        let styles += [ item[31:-5] ]
    endfor

    return styles
endfunction

let s:style_list = s:get_styles()

function! s:complete(lead, line, pos)

    let items = []

    for item in s:style_list
        "if item =~ a:lead
        if match(item, a:lead) == 0
            let items += [ item ]
        endif
    endfor

    return items
endfunction

CursorTooltipStyle nord
