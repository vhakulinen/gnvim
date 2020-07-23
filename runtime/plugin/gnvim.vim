if has("g:gnvim_runtime_loaded")
    finish
endif

let g:gnvim_runtime_loaded = 1

inoremap <expr> <C-s> gnvim#popupmenu#toggle_details()

command! -nargs=1 GnvimCursorEnableAnimations
            \ call gnvim#cursor#enable_animations(<q-args>)
