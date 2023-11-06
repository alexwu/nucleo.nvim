local Entry = require("plenary.class"):extend()

function Entry:new(entry)
	self.entry = entry
	self.icon = require("nvim-web-devicons").get_icon(entry.path, entry.file_type, { default = true })
end

---@return string
function Entry:render()
	return table.concat({
		self.icon,
		self.entry.path,
	}, " ")
end

return Entry
