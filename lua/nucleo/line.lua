local NuiLine = require("nui.line")

---@class Line: NuiLine
local Line = NuiLine:extend("Line")

---@class LineOptions
---@field separator? string

---@param texts string|NuiText[]
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

---@param texts string|NuiText[]
function Line:set(texts)
	self._texts = texts
end

-- TODO: Make this take a line number
-- Perhaps a line will always correspond to a specific line number for the existence of the picker?
-- Then we push new strings + indices into the line?
-- Only rerender if the content is different? Will need to account for the icon somehow though
-- Not sure what will happen when the number of lines change
-- Also perhaps i will make it so this follows the NoiceChunk stuff?
-- Perhaps I can do partial edits too idk

return Line
