function! gnvim#enable_ext_tabline(enable)
    return rpcnotify(
                \ g:gnvim_channel_id,
                \ 'Gnvim',
                \ 'EnableExtTabline',
                \ a:enable)
endfunction

function! gnvim#enable_ext_cmdline(enable)
    return rpcnotify(
                \ g:gnvim_channel_id,
                \ 'Gnvim',
                \ 'EnableExtCmdline',
                \ a:enable)
endfunction

function! gnvim#enable_ext_popupmenu(enable)
    return rpcnotify(
                \ g:gnvim_channel_id,
                \ 'Gnvim',
                \ 'EnableExtPopupmenu',
                \ a:enable)
endfunction
