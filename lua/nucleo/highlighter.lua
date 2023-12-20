---@class Nucleo.Highlighter: Object
---@field picker PickerBackend
---@field results Nucleo.Results
---@field bufnr number
local Highlighter = require("plenary.class"):extend()
local log = require("nucleo.log")
local api = vim.api

local ns_selection = vim.api.nvim_create_namespace("nucleo_selection")

---@param opts Nucleo.Config.Highlighter
function Highlighter:new(opts)
	opts = vim.F.if_nil(opts, {})
	vim.validate({
		picker = { opts.picker, "userdata" },
		results = { opts.results, "table" },
	})

	self.picker = opts.picker
	self.results = opts.results
	self.caret_extmark_id = nil
end

---@param highlighter Nucleo.Highlighter
---@param picker PickerBackend
---@param results Nucleo.Results
local highlight_selection = function(highlighter, picker, results)
	api.nvim_buf_clear_namespace(results.bufnr, ns_selection, 0, -1)

	if picker:total_matches() == 0 then
		return
	end

	local line_nr = picker:get_cursor_pos()
	if not line_nr then
		return
	end

	if picker:sort_direction() == "ascending" then
		local height = api.nvim_win_get_height(results.winid)
		line_nr = height - line_nr - 1
	end

	log.info("highlight_selection", line_nr)
	log.info("buf_line_count: ", api.nvim_buf_line_count(results.bufnr))

	local selection_line = api.nvim_buf_get_lines(results.bufnr, line_nr, line_nr + 1, false)

	if vim.tbl_isempty(selection_line) or #selection_line[1] == 0 then
		return
	end

	highlighter.caret_extmark_id = api.nvim_buf_set_extmark(results.bufnr, ns_selection, line_nr, 0, {
		id = highlighter.caret_extmark_id,
		hl_eol = false,
		virt_text_win_col = 0,
		virt_text = { { ">", "TelescopeSelectionCaret" } },
	})

	api.nvim_buf_set_extmark(results.bufnr, ns_selection, line_nr, 1, {
		hl_eol = true,
		end_row = line_nr + 1,
		hl_group = "TelescopeSelection",
	})
end

function Highlighter:highlight_selection()
	highlight_selection(self, self.picker, self.results)
end

return Highlighter
