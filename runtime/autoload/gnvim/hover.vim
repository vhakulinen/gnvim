
let s:gnvim_hover_timer = 0
let s:gnvim_hover_pos = 0

function! gnvim#hover#cursor_moved()
    let pos = getcurpos()
    let cur_row = l:pos[1] - 1
    let cur_col = l:pos[2] - 1

    if type(s:gnvim_hover_pos) == type({})
        let start = s:gnvim_hover_pos['start']
        let end = s:gnvim_hover_pos['end']

        let after_start = v:false
        let before_end = v:false

        if cur_row == l:start['line']
            if cur_col >= start['character']
                let after_start = v:true
            endif
        endif

        if cur_row == end['line']
            if cur_col <= end['character']
                let before_end = v:true
            endif
        endif

        if after_start && before_end
            return
        endif

        " TODO(ville): Check if cur_row < end['line'] && cur_row > start['line']
    endif

    call gnvim#cursor_tooltip#hide()
    let s:gnvim_hover_pos = 0

    if s:gnvim_hover_timer != 0
        call timer_stop(s:gnvim_hover_timer)
    endif

    let s:gnvim_hover_timer = timer_start(300, function('s:timer_cb'))
endfunction

function! s:timer_cb(id)
    call gnvim#hover#show_hover()
endfunction

function! gnvim#hover#abort()
    call timer_stop(s:gnvim_hover_timer)
    let s:gnvim_hover_pos = 0
    call gnvim#cursor_tooltip#hide()
endfunction

function! gnvim#hover#show_hover() abort
    let l:servers = filter(lsp#get_whitelisted_servers(), 'lsp#capabilities#has_hover_provider(v:val)')

    if len(l:servers) == 0
        return
    endif

    let l:pos = lsp#get_position()
    let l:screencol = screencol()
    let l:screenrow = screenrow()

    for l:server in l:servers
        call lsp#send_request(l:server, {
            \ 'method': 'textDocument/hover',
            \ 'params': {
            \   'textDocument': lsp#get_text_document_identifier(),
            \   'position': l:pos,
            \ },
            \ 'on_notification': function('s:handle_hover', [l:server, l:pos, l:screencol, l:screenrow]),
            \ })
    endfor
endfunction

function! s:handle_hover(server, pos, screencol, screenrow, data) abort
    if lsp#client#is_error(a:data['response'])
        call lsp#utils#error('Failed to retrieve hover information for ' . a:server)
        return
    endif

    if !has_key(a:data['response'], 'result')
        return
    endif

    if !empty(a:data['response']['result']) && !empty(a:data['response']['result']['contents'])

        let l:pos = [ a:pos['line'], a:pos['character'] ]

        " If the current cursor location is not the same position that got
        " passed as argument, that means that the user has moved the cursor
        " before _this_ request could complete. Cancel the request.
        if l:pos[0] != line('.') - 1
            return
        elseif l:pos[1] != col('.') - 1
            " TODO(ville): If we're inside the 'range' that is in
            " a:data['response']['result']['range'], we should not return here.
            return
        endif

        " Get the offset of cursor location related to the whole screen.
        " There are things like the line number column, window splits etc.
        " that is not included in buffer cursor locations.
        let col_offset = a:screencol - 1 - l:pos[1]

        if has_key(a:data['response']['result'], 'range')
            let s:gnvim_hover_pos = a:data['response']['result']['range']
            let l:pos[0] = s:gnvim_hover_pos['start']['line']
            let l:pos[1] = s:gnvim_hover_pos['start']['character']
        else
            let s:gnvim_hover_pos = 0
        endif

        let l:content = s:to_string(a:data['response']['result']['contents'])

        call gnvim#cursor_tooltip#show(l:content, a:screenrow - 1, max([l:pos[1] + col_offset, 0]))
    endif
endfunction

function! s:to_string(data) abort
    let l:content = ""

    if type(a:data) == type([])
        for l:entry in a:data
            let l:content .= s:to_string(entry)
        endfor
    elseif type(a:data) == type('')
        let l:content .= a:data . "\n"
    elseif type(a:data) == type({}) && has_key(a:data, 'language')
        let l:content .= "```".a:data.language . "\n"
        let l:content .= a:data.value
        let l:content .= "\n" . "```" . "\n"

    elseif type(a:data) == type({}) && has_key(a:data, 'kind')
        let l:content .= a:data.value . "\n"
    endif

    return l:content
endfunction
