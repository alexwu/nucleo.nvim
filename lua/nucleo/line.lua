local NuiLine = require("nui.line")

---@class Line: NuiLine
---@field super NuiLine
---@field _texts string[]|NuiText[]
local Line = NuiLine:extend("Line")

---@class LineOptions
---@field separator? string

---@param texts string[]|NuiText[]
---@param options? LineOptions
function Line:init(texts, options)
	local opts = options or {}
	self.separator = opts.separator or " "

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

---@param texts string[]|NuiText[]
function Line:set(texts)
	self._texts = texts
end

return Line
