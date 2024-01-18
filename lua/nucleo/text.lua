local NuiText = require("nui.text")
local log = require("nucleo.log")

---@type integer
local fallback_namespace_id = vim.api.nvim_create_namespace("nucleo.nvim")

---@private
---@param ns_id integer
---@return integer
local function ensure_namespace_id(ns_id)
	return ns_id == -1 and fallback_namespace_id or ns_id
end

---@class Text: NuiText
---@field super NuiText
---@field extmarks nui_text_extmark[]
---@diagnostic disable-next-line: undefined-field
local Text = NuiText:extend("Text")

---@class TextOptions
---@field separator? string

-- ---@param content string|NuiText text content or NuiText object
-- ---@param extmark? string|nui_text_extmark highlight group name or extmark options
-- function Text:init(content, extmark)
-- 	Text.super.init(self, content, extmark)
-- end

---@param content string|Text|NuiText text content or NuiText object
---@param extmarks? string|nui_text_extmark highlight group name or extmark options
function Text:init(content, extmarks)
	self.extmarks = {}
	local extmark
	if type(extmarks) == "string" then
		extmark = { extmarks }
	else
		extmark = extmarks
	end

	if type(content) == "string" then
		self:set(content, extmark)
	else
		-- cloning
		self:set(content._content, extmarks or content.extmarks)
	end
end

---@param content string text content
---@param extmarks? string|nui_text_extmark highlight group name or extmark options
---@return NuiText
function Text:set(content, extmarks)
	if self._content ~= content then
		self._content = content
		self._length = vim.fn.strlen(content)
		self._width = vim.api.nvim_strwidth(content)
	end

	if extmarks then
		vim.iter(ipairs(extmarks)):each(function(i, extmark)
			-- preserve self.extmark.id
			local id = self.extmarks[i] and self.extmarks[i].id or nil

			if type(extmark) == "string" then
				self.extmarks[i] = { opts = { hl_group = extmark, id = id } }
			else
				self.extmarks[i] = vim.deepcopy(extmark)
			end

			self.extmarks[i].id = id
		end)
	end

	return self
end

---@param bufnr number buffer number
---@param ns_id number namespace id
---@param linenr number line number (1-indexed)
---@param byte_start number start byte position (0-indexed)
---@return nil
function Text:highlight(bufnr, ns_id, linenr, byte_start)
	if not self.extmarks or vim.tbl_isempty(self.extmarks) then
		return
	end

	for i = 1, #self.extmarks do
		if not self.extmarks[i].start_col then
			self.extmarks[i].start_col = byte_start
		else
			self.extmarks[i].start_col = self.extmarks[i].start_col + byte_start
		end

		if not self.extmarks[i].opts.end_col then
			self.extmarks[i].opts.end_col = byte_start + self:length()
		else
			self.extmarks[i].opts.end_col = self.extmarks[i].opts.end_col + byte_start
		end

		local extmark = vim.deepcopy(self.extmarks[i])
		extmark.opts.id = self.extmarks[i].id

		self.extmarks[i].id = vim.api.nvim_buf_set_extmark(
			bufnr,
			self.extmarks[i].ns_id or ensure_namespace_id(ns_id),
			linenr - 1,
			self.extmarks[i].start_col,
			extmark.opts
		)
	end
end

return Text
