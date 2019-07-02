function! gnvim#tabline#enable(bool)
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'EnableExtTabline', a:bool)
    return ''
endfunction
