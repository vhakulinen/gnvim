function! gnvim#cursor#enable_animations(enable)
    return rpcnotify(
                \ g:gnvim_channel_id,
                \ 'Gnvim',
                \ 'EnableCursorAnimations',
                \ a:enable == 1)
endfunction
