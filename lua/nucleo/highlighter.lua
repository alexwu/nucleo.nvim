---@class Highlighter
---@field picker Picker
---@field results Results
---@field bufnr number
local Highlighter = require("plenary.class"):extend()
local api = vim.api
local log = require("nucleo.log")

local ns_selection = vim.api.nvim_create_namespace("nucleo_selection")

---@param opts HighlighterConfig
function Highlighter:new(opts)
	opts = vim.F.if_nil(opts, {})
	vim.validate({
		picker = { opts.picker, "userdata" },
		results = { opts.results, "table" },
	})

	self.picker = opts.picker
	self.results = opts.results
end

---@param highlighter Highlighter
local highlight_selection = function(highlighter)
	api.nvim_buf_clear_namespace(highlighter.results.bufnr, ns_selection, 0, -1)

	if highlighter.picker:total_matches() == 0 then
		return
	end

	local line_nr = highlighter.picker:get_selection_index()
	if highlighter.results.sort_direction == "ascending" then
		local height = vim.api.nvim_win_get_height(highlighter.results.winid)
		line_nr = height - line_nr - 1
	end

	api.nvim_buf_set_extmark(highlighter.results.bufnr, ns_selection, line_nr, 0, {
		hl_eol = false,
		virt_text_win_col = 0,
		virt_text = { { ">", "TelescopeSelectionCaret" } },
	})

	log.info("highlight_selection", line_nr)
	api.nvim_buf_set_extmark(
		highlighter.results.bufnr,
		ns_selection,
		line_nr,
		1,
		{ hl_eol = true, hl_group = "TelescopeSelection" }
	)
end

function Highlighter:highlight_selection()
	highlight_selection(self)
end

return Highlighter
