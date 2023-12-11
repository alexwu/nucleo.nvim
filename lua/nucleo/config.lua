local actions = require("nucleo.actions")
local extensions = require("nucleo.extensions")

---@class Nucleo.Config
local M = {}

---@class Nucleo.Keymap
---@field [integer] function
---@field opts table

---@class Nucleo.Config.Values
local defaults = {
	defaults = {
		sort_direction = "descending",
	},
	sources = {
		---@type Nucleo.FilePicker.Config
		files = {
			cwd = vim.uv.cwd,
			git_ignore = true,
			ignore = true,
		},
	},
	---@class Nucleo.Keymaps
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
			["<ScrollWheelUp>"] = {
				actions.scroll_up,
				opts = {
					desc = "Scroll up in results list",
				},
			},
			["<ScrollWheelDown>"] = {
				actions.scroll_down,
				opts = {
					desc = "Scroll down in results list",
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
			["<C-r>"] = {
				actions.force_refresh,
				opts = {
					desc = "Force refresh the picker",
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
			["<C-v>"] = {
				actions.open_in_vsplit,
				opts = {
					desc = "Open current selection in vertical split",
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

---@param source string
---@param ... string|nil
function M.get_source_config(source, ...)
	M.get("sources", source, ...)
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
