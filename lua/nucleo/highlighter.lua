local Highlighter = require("plenary.class"):extend()
local api = vim.api

local ns_selection = vim.api.nvim_create_namespace("nucleo_selection")

function Highlighter:new(opts)
	opts = vim.F.if_nil(opts, {})
	vim.validate({
		picker = { opts.picker, "userdata" },
		bufnr = { opts.bufnr, "number" },
	})

	self.picker = opts.picker
	self.bufnr = opts.bufnr
end

function Highlighter:highlight_selection()
	api.nvim_buf_clear_namespace(self.bufnr, ns_selection, 0, -1)
	-- api.nvim_buf_add_highlight(self.bufnr, ns_selection, "TelescopeSelectionCaret", row, 0, #caret)

	local line_nr = self.picker:get_selection_index()
	api.nvim_buf_set_extmark(
		self.bufnr,
		ns_selection,
		line_nr,
		1,
		{ end_row = line_nr + 1, hl_eol = true, hl_group = "TelescopeSelection" }
	)
end

return Highlighter
