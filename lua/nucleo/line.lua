local NuiLine = require("nui.line")

---@class Nucleo.Line: NuiLine
---@field super NuiLine
---@field _texts Text[]
---@diagnostic disable-next-line: undefined-field
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
	self._last_content = self:content()
	self._texts = texts
end

---@param bufnr number buffer number
---@param ns_id number namespace id
---@param linenr number line number (1-indexed)
---@param ___byte_start___? integer start byte position (0-indexed)
---@return nil
function Line:highlight(bufnr, ns_id, linenr, ___byte_start___)
	local current_byte_start = ___byte_start___ or 0
	for _, text in ipairs(self._texts) do
		text:highlight(bufnr, ns_id, linenr, current_byte_start)
		current_byte_start = current_byte_start + text:length() + vim.fn.strlen(self.separator)
	end
end

---@param bufnr number buffer number
---@param ns_id number namespace id
---@param linenr_start number start line number (1-indexed)
---@param linenr_end? number end line number (1-indexed)
---@return nil
function Line:render(bufnr, ns_id, linenr_start, linenr_end)
	local row_start = linenr_start - 1
	local row_end = linenr_end and linenr_end - 1 or row_start + 1
	local content = self:content()
	if #content == 0 or self._last_content ~= content then
		vim.api.nvim_buf_set_lines(bufnr, row_start, row_end, false, { content })
	end
	self:highlight(bufnr, ns_id, linenr_start)
end

return Line
