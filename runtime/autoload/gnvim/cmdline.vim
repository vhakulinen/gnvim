function! gnvim#cmdline#enable(bool)
    call rpcnotify(g:gnvim_channel_id, 'Gnvim', 'EnableExtCmdline', a:bool)
    return ''
endfunction
