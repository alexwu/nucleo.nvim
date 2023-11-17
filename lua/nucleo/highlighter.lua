---@class Highlighter
---@field picker Picker
---@field bufnr number
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

---@param highlighter Highlighter
local highlight_selection = function(highlighter)
	api.nvim_buf_clear_namespace(highlighter.bufnr, ns_selection, 0, -1)

	local line_nr = highlighter.picker:get_selection_index()

	api.nvim_buf_set_extmark(highlighter.bufnr, ns_selection, line_nr, 0, {
		end_col = 1,
		hl_eol = false,
		virt_text_pos = "overlay",
		virt_text = { { ">", "TelescopeSelectionCaret" } },
	})

	api.nvim_buf_set_extmark(
		highlighter.bufnr,
		ns_selection,
		line_nr,
		1,
		{ end_row = line_nr + 1, hl_eol = true, hl_group = "TelescopeSelection" }
	)
end

function Highlighter:highlight_selection()
	highlight_selection(self)
end

return Highlighter
