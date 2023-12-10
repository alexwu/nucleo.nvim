local actions = require("nucleo.actions")
local extensions = require("nucleo.extensions")

---@class Nucleo.Config
local M = {}

---@enum Mode ''

---@class Nucleo.Keymaps
---@field table<, Nucleo.Keymap>

---@class Nucleo.Keymap
---@field [integer] function
---@field opts table

---@class Nucleo.Config.Values
local defaults = {
	sort_direction = "descending",
	mappings = {
		i = {
			---@type table<string, Nucleo.Keymap>
			["<C-p>"] = {
				actions.move_cursor_up,
				opts = {
					desc = "Move up in results list",
				},
			},
			["<Up>"] = {
				actions.move_cursor_up,
				opts = {
					desc = "Move up in results list",
				},
			},
			["<C-n>"] = {
				actions.move_cursor_down,
				opts = {
					desc = "Move down in results list",
				},
			},
			["<Down>"] = {
				actions.move_cursor_down,
				opts = {
					desc = "Move down in results list",
				},
			},
			["<C-b>"] = {
				actions.move_to_top,
				opts = {
					desc = "Move to the top of the results list",
				},
			},
			["<C-f>"] = {
				actions.move_to_bottom,
				opts = {
					desc = "Move to the bottom of the results list",
				},
			},
			["<C-s>"] = {
				extensions.flash.jump,
				opts = {
					desc = "Jump to a result",
				},
			},
			["<Tab>"] = {
				actions.multiselect,
				opts = {
					desc = "Multi-select the current selection",
				},
			},
			["<Esc>"] = {
				actions.close,
				opts = {
					desc = "Close the picker",
				},
			},
		},
		n = {
			["<Esc>"] = {
				actions.close,
				opts = {
					desc = "Close the picker",
				},
			},
		},
	},
}

---@type Nucleo.Config.Values
local options

---@param opts? Nucleo.Config.Values
function M.setup(opts)
	opts = opts or {}

	local all = { {}, defaults, options or {} }

	---@cast options Nucleo.Config.Values
	options = vim.tbl_deep_extend("force", unpack(all))
end

---@param ... string|nil
---@return Nucleo.Config.Values
function M.get(...)
	if options == nil then
		M.setup()
	end

	---@diagnostic disable-next-line: param-type-mismatch
	return vim.tbl_get(options, ...)
end

return setmetatable(M, {
	__index = function(_, key)
		if options == nil then
			M.setup()
		end
		---@diagnostic disable-next-line: need-check-nil
		return options[key]
	end,
})
