---@class Nucleo.Entry: Object
---@field index number
---@field bufnr number
---@field selection_caret string
---@field icon { value: string, color?: string }
local Entry = require("plenary.class"):extend()
local Line = require("nucleo.line")
local Text = require("nucleo.text")
local devicons = require("nvim-web-devicons")
local align_str = require("nucleo_rs").align_str
local api = vim.api

local ns_matching = vim.api.nvim_create_namespace("nucleo_matching")

---@class Nucleo.Picker.Entry
---@field value table
---@field preview_options? table

---@param index number Lua index-ed
function Entry:new(index, entry, bufnr, ns_multiselection_id, winid)
	self.index = index
	self.entry = entry
	self.bufnr = bufnr
	self.selection_caret = " "
	self.selection_caret_extmark_id = nil
	self.ns_multiselection_id = ns_multiselection_id
	self.match_extmarks = {}
	self.winid = winid
	self.line = Line({})

	self:update_icon()
end

function Entry:update_icon()
	if self.entry and self.entry.preview_options and self.entry.preview_options.file_extension then
		local value, color = devicons.get_icon(self.entry.value.path, self.entry.value.file_extension, { default = true })
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
	local truncated_text, indices = unpack(align_str(self.entry.ordinal, self.entry.indices, width, "…", 10))
	local extmarks = vim.iter(indices)
		:map(function(range)
			local start_col, end_col = unpack(range)

			return {
				start_col = start_col,
				ns_id = ns_matching,
				opts = {
					hl_eol = false,
					end_col = end_col + 1,
					hl_group = "TelescopeMatching",
				},
			}
		end)
		:totable()

	-- TODO: Figure out a way so update the highlights instead of clearing them every time
	api.nvim_buf_clear_namespace(self.bufnr, ns_matching, self.index - 1, self.index)

	local path = Text(truncated_text, extmarks)

	self.line:set({ picker_icon, icon, path })
	self.line:render(self.bufnr, -1, self.index)

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
