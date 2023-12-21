---@class Nucleo.Entry: Object
---@field index number
---@field bufnr number
---@field selection_caret string
---@field icon { value: string, color?: string }
local Entry = require("plenary.class"):extend()
local Line = require("nucleo.line")
local Text = require("nucleo.text")
local devicons = require("nvim-web-devicons")
local align_str = require("nucleo_rs").align_str
local api = vim.api

local ns_matching = vim.api.nvim_create_namespace("nucleo_matching")

---@class Nucleo.Picker.Entry
---@field value table
---@field preview_options? table

---@param index number Lua index-ed
function Entry:new(index, entry, bufnr, ns_multiselection_id, winid)
	self.index = index
	self.entry = entry
	self.bufnr = bufnr
	self.selection_caret = " "
	self.selection_caret_extmark_id = nil
	self.ns_multiselection_id = ns_multiselection_id
	self.match_extmarks = {}
	self.winid = winid
	self.line = Line({})

	self:update_icon()
end

function Entry:update_icon()
	if self.entry and self.entry.preview_options and self.entry.preview_options.file_type then
		local value, color = devicons.get_icon(self.entry.value.path, self.entry.value.file_type, { default = true })
		self.icon = {
			value = value,
			color = color,
		}
	else
		self.icon = {
			value = " ",
			color = nil,
		}
	end
end

function Entry:update(entry)
	self.entry = entry
	self:update_icon()
end

function Entry:render()
	if not self.entry then
		self.line:set({})
		self.line:render(self.bufnr, -1, self.index)

		return
	end

	local picker_icon = Text(self.selection_caret, "Normal")
	local icon = Text(self.icon.value, self.icon.color)
	local leading_length = picker_icon:length() + icon:length()

	local width = api.nvim_win_get_width(self.winid) - leading_length
	local truncated_text, indices = unpack(align_str(self.entry.ordinal, self.entry.indices, width, "…", 10))
	local path = Text(truncated_text)

	local last_line_content = self.line:content()

	self.line:set({ picker_icon, icon, path })

	if last_line_content ~= self.line:content() then
		self.line:render(self.bufnr, -1, self.index)
	end

	if vim.tbl_isempty(indices) then
		api.nvim_buf_clear_namespace(self.bufnr, ns_matching, self.index - 1, self.index)
	end

	vim.iter(ipairs(indices)):each(function(i, range)
		local start_col = leading_length + range[1] + 2
		local end_col = leading_length + range[2] + 2

		if start_col > 0 and end_col > 0 then
			self.match_extmarks[i] = api.nvim_buf_set_extmark(self.bufnr, ns_matching, self.index - 1, start_col, {
				id = self.match_extmarks[i],
				hl_eol = false,
				end_col = end_col + 1,
				hl_group = "TelescopeMatching",
			})
		end
	end)

	if self.entry.selected then
		self.selection_caret_extmark_id =
			api.nvim_buf_set_extmark(self.bufnr, self.ns_multiselection_id, self.index - 1, 0, {
				id = self.selection_caret_extmark_id,
				hl_eol = false,
				virt_text_win_col = 0,
				virt_text = { { "+", "TelescopeMultiSelection" } },
			})
	else
		self.selection_caret_extmark_id = nil
	end
end

return Entry
