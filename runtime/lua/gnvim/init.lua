local M = {}

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

return M
