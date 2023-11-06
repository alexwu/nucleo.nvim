local Entry = require("plenary.class"):extend()

function Entry:new(entry)
	self.entry = entry
	self.icon = require("nvim-web-devicons").get_icon(entry.path, entry.file_type, { default = true })

	if self.entry.selected then
		self.picker_icon = ""
	else
		self.picker_icon = " "
	end
end

---@return string
function Entry:render()
	return table.concat({
		self.picker_icon,
		self.icon,
		self.entry.path,
	}, " ")
end

return Entry
