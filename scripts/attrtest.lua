-- Basic test for rendering font attributes.

local ns = vim.api.nvim_create_namespace('attrtest')

vim.api.nvim_set_hl(0, 'AttrTestFg', {
  fg = 'red'
})

vim.api.nvim_set_hl(0, 'AttrTestBg', {
  fg = 'black',
  bg = 'green'
})

vim.api.nvim_set_hl(0, 'AttrTestUnderline', {
  underline = true,
  sp = 'cyan',
})

vim.api.nvim_set_hl(0, 'AttrTestUnderlineline', {
  underdouble = true,
  sp = 'cyan',
})

vim.api.nvim_set_hl(0, 'AttrTestStrike', {
  strikethrough = true,
  sp = 'cyan',
})

vim.api.nvim_set_hl(0, 'AttrTestUndercurl', {
  undercurl = true,
  sp = 'cyan',
})

vim.api.nvim_set_hl(0, 'AttrTestUnderdot', {
  underdotted = true,
  sp = 'cyan',
})

vim.api.nvim_set_hl(0, 'AttrTestUnderdash', {
  underdashed = true,
  sp = 'cyan',
})

local bufno = vim.api.nvim_create_buf(true, false)

vim.api.nvim_buf_set_lines(
  bufno,
  0,
  -1,
  false,
  {
    'foreground line',
    'background line',
    'underline line',
    'underlineline line',
    'strikethrough line',
    'underdot line',
    'underdash line',
    'undercurl line',
  }
)

vim.api.nvim_buf_set_option(bufno, 'buftype', 'nowrite')

vim.highlight.range(bufno, ns, 'AttrTestFg', {0, 0}, {0, -1})
vim.highlight.range(bufno, ns, 'AttrTestBg', {1, 0}, {1, -1})
vim.highlight.range(bufno, ns, 'AttrTestUnderline', {2, 0}, {2, -1})
vim.highlight.range(bufno, ns, 'AttrTestUnderlineline', {3, 0}, {3, -1})
vim.highlight.range(bufno, ns, 'AttrTestStrike', {4, 0}, {4, -1})
vim.highlight.range(bufno, ns, 'AttrTestUnderdot', {5, 0}, {5, -1})
vim.highlight.range(bufno, ns, 'AttrTestUnderdash', {6, 0}, {6, -1})
vim.highlight.range(bufno, ns, 'AttrTestUndercurl', {7, 0}, {7, -1})

vim.api.nvim_win_set_buf(0, bufno)
