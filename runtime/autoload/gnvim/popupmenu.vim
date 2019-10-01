function! gnvim#popupmenu#set_width(width)
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'PopupmenuSetWidth', a:width)
endfunction

function! gnvim#popupmenu#set_width_details(width)
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'PopupmenuSetWidthDetails', a:width)
endfunction

function! gnvim#popupmenu#toggle_details()
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'CompletionMenuToggleInfo')
    return ''
endfunction

function! gnvim#popupmenu#show_menu_on_all_items(bool)
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'PopupmenuShowMenuOnAllItems', a:bool)
    return ''
endfunction
