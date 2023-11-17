local NuiLine = require("nui.line")

---@class Line: NuiLine
local Line = NuiLine:extend("Line")

-- TODO: Make the separator a part of the Line API
function Line:init(texts, options)
	self.separator = " "

	Line.super.init(self, texts)
end

---@return string
function Line:content()
	return table.concat(
		vim.tbl_map(function(text)
			return text:content()
		end, self._texts),
		self.separator
	)
end

return Line
