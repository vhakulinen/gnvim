local M = { popupmenu = {} }

function M.find_gnvim_chans()
  local nvim_chans = {}
  local chans = vim.api.nvim_list_chans();

  for _, chan in pairs(chans) do
    local is_gnvim = vim.tbl_get(chan, 'client', 'name') == 'gnvim'

    if is_gnvim then
      table.insert(nvim_chans, chan['id'])
    end
  end

  -- TODO(ville): Display error of no gnvim channels found?

  return nvim_chans
end

--- Send notification to avaialble gnvim GUIs.
---
---@param fn Function to call
---@param ... Arguments for fn
function M.notify(fn, ...)
  for _, chan in ipairs(M.find_gnvim_chans()) do
    vim.rpcnotify(chan, "gnvim", {
      ['fn'] = fn,
      ['args'] = ...,
    })
  end
end

function M.echo_repeat(msg, times)
  M.notify('echo_repeat', {
    msg = msg,
    times = times,
  })
end

function M.gtk_debugger()
  M.notify('gtk_debugger')
end

function M.setup(opts)
  M.notify('setup', opts)
end

function M.popupmenu.kind(label, hl)
  local adapt = function(hl, normal)
    local attrs = vim.api.nvim_get_hl(0, { name = hl })
    local bg
    if normal then
        bg = vim.api.nvim_get_hl(0, { name = "Pmenu" }).bg
    else
        bg = vim.api.nvim_get_hl(0, { name = "PmenuSel" }).bg
    end

    return {
      fg = attrs.fg,
      bg = bg,
      italic = attrs.italic,
      bold = attrs.bold,
    }
  end

  return {
    label = label,
    hl = adapt(hl, true),
    sel_hl = adapt(hl, false),
  }
end

return M
