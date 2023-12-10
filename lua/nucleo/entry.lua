---@class Entry: Object
---@field index number
---@field bufnr number
---@field selection_caret string
---@field icon { value: string, color: string }
local Entry = require("plenary.class"):extend()
local Line = require("nucleo.line")
local Text = require("nui.text")

local ns_matching = vim.api.nvim_create_namespace("nucleo_matching")

---@param index number Lua index-ed
function Entry:new(index, entry, bufnr)
	self.index = index
	self.entry = entry
	self.bufnr = bufnr
	self.selection_caret = " "

	local value, color = require("nvim-web-devicons").get_icon(entry.path, entry.file_type, { default = true })
	self.icon = {
		value = value,
		color = color,
	}
end

function Entry:render()
	local picker_icon = Text(self.selection_caret, "Normal")
	local icon = Text(self.icon.value, self.icon.color)
	local path = Text(self.entry.match_value)
	local line = Line({ picker_icon, icon, path })

	local leading_length = picker_icon:length() + icon:length()

	line:render(self.bufnr, -1, self.index)
	vim.iter(self.entry.indices):each(function(range)
		vim.highlight.range(
			self.bufnr,
			ns_matching,
			"TelescopeMatching",
			{ self.index - 1, leading_length + 1 + range[1] + 1 },
			{ self.index - 1, leading_length + 1 + range[2] + 1 },
			{ inclusive = true }
		)
	end)
end

return Entry
