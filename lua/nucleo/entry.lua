---@class Nucleo.Entry: Object
---@field index number
---@field bufnr number
---@field selection_caret string
---@field icon { value: string, color: string }
local Entry = require("plenary.class"):extend()
local Line = require("nucleo.line")
local Text = require("nucleo.text")
local devicons = require("nvim-web-devicons")
local strings = require("plenary.strings")
local api = vim.api

local ns_matching = vim.api.nvim_create_namespace("nucleo_matching")

---@class Nucleo.Picker.Entry
---@field path string
---@field file_type string

---@param index number Lua index-ed
function Entry:new(index, entry, bufnr, ns_multiselection_id, winid)
	self.index = index
	self.entry = entry
	self.bufnr = bufnr
	self.selection_caret = " "
	self.selection_caret_extmark_id = nil
	self.ns_multiselection_id = ns_multiselection_id
	self.winid = winid
	self.line = Line({})

	self:update_icon()
end

function Entry:update_icon()
	if self.entry and self.entry.value.file_type then
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
	local truncated_text = strings.truncate(self.entry.display, width, nil, -1)
	local path = Text(truncated_text)

	self.line:set({ picker_icon, icon, path })

	local truncation_offset = vim.fn.strlen(self.entry.display) - vim.fn.strlen(truncated_text)
	self.line:render(self.bufnr, -1, self.index)

	vim.iter(self.entry.indices):each(function(range)
		local start_col = leading_length + 1 + range[1] + 1 - truncation_offset
		local end_col = leading_length + 1 + range[2] + 1 - truncation_offset

		if start_col > 0 and end_col > 0 then
			vim.highlight.range(
				self.bufnr,
				ns_matching,
				"TelescopeMatching",
				{ self.index - 1, start_col },
				{ self.index - 1, end_col },
				{ inclusive = true }
			)
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
