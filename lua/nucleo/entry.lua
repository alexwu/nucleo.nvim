local Entry = require("plenary.class"):extend()
local Line = require("nui.line")
local Text = require("nui.text")

local ns_selection = vim.api.nvim_create_namespace("nucleo_selection")

function Entry:new(index, entry, bufnr)
	self.index = index
	self.entry = entry
	self.bufnr = bufnr
	local value, color = require("nvim-web-devicons").get_icon(entry.path, entry.file_type, { default = true })
	self.icon = {
		value = value,
		color = color,
	}

	if self.entry.selected then
		self.picker_icon = ""
	else
		self.picker_icon = " "
	end
end

---@return string
function Entry:render()
	local line = Line()
	local picker_icon = Text(self.picker_icon, "Comment")
	line:append(picker_icon)
	line:append(" ")
	local icon = Text(self.icon.value, self.icon.color)
	line:append(icon)
	line:append(" ")
	local path = Text(self.entry.path, "Normal")
	line:append(path)

	-- if self.entry:get_selection_index() == self.index - 1 then
	-- 	line:highlight("TelescopeSelection")
	-- end
	--
	-- return table.concat({
	-- 	self.picker_icon,
	-- 	self.icon.value,
	-- 	self.entry.path,
	-- }, " ")
	line:render(self.bufnr, -1, self.index, -1)
end

return Entry
