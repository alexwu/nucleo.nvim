local Entry = require("nucleo.entry")
local NuiPopup = require("nui.popup")
local log = require("nucleo.log")
local api = vim.api

local ns_multiselection = vim.api.nvim_create_namespace("nucleo_multiselection")

---@class Nucleo.Results: NuiPopup
---@field super NuiPopup
---@field _entries Nucleo.Entry[]
---@diagnostic disable-next-line: undefined-field
local Results = NuiPopup:extend("Results")

---@class ResultsOptions
---@field popup_options? nui_popup_options

---@param opts? ResultsOptions
function Results:init(opts)
	self._entries = {}
	opts = vim.F.if_nil(opts, { popup_options = {} })
	local popup_options = vim.tbl_deep_extend("force", opts.popup_options or {}, {
		border = "rounded",
		focusable = true,
		buf_options = {
			filetype = "NucleoResults",
		},
		win_options = {
			winhighlight = "Normal:Normal,FloatBorder:FloatBorder",
		},
		options = {},
	})

	Results.super.init(self, popup_options)
end

function Results:clear_buffer()
	vim.iter(self._entries):each(function(entry)
		entry:update(nil)
		entry:render()
	end)
end

---@param picker PickerBackend
function Results:render_entries(picker)
	if not self.winid then
		return
	end
	local height = vim.api.nvim_win_get_height(self.winid)

	if vim.tbl_isempty(self._entries) then
		log.info("Trying to fill buffer to height: ", height)
		for i = 1, height do
			self._entries[i] = Entry(i, nil, self.bufnr, ns_multiselection, self.winid)
			self._entries[i]:render()
		end
		log.info("entries: ", #self._entries)
		log.info("buf_line_count: ", vim.api.nvim_buf_line_count(self.bufnr))
	end

	api.nvim_buf_clear_namespace(self.bufnr, ns_multiselection, 0, -1)
	if picker:total_matches() == 0 then
		if vim.api.nvim_buf_is_loaded(self.bufnr) and vim.api.nvim_win_is_valid(self.winid) then
			self:clear_buffer()
		end
	else
		log.info("trying to render index at height: ", height)
		local results = picker:current_matches()

		for i = 1, height do
			local index = i
			if picker:sort_direction() == "ascending" then
				index = height - i + 1
				log.info("trying to render index", index)
			end

			local result = nil
			-- if index <= #results then
			-- 	result = results[index]
			-- end
			if i <= #results then
				result = results[i]
			end

			if not self._entries[index] then
				self._entries[index] = Entry(index, result, self.bufnr, ns_multiselection, self.winid)
			else
				self._entries[index]:update(result)
			end

			self._entries[index]:render()
		end
	end
end

return Results
