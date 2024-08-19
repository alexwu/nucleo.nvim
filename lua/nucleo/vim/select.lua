---@param item any
---@param format_item fun(item: any): string
---@return table
local function make_data(item, format_item)
	return {
		ordinal = format_item(item),
		score = 0,
		kind = "lua",
		selected = false,
		indices = {},
		value = item,
	}
end

---@param items any[]
---@param opts table
---@param on_choice fun(item: any?, idx: integer?)
return function(items, opts, on_choice)
	local Picker = require("nucleo.picker")

	local format_item = opts.format_item or function(item)
		return item
	end

	local title = opts.prompt or ""

	Picker({
		source = {
			name = "custom.ui.select",
			config = {},
			finder = function()
				return vim.iter(items)
					:map(function(item)
						return make_data(item, format_item)
					end)
					:totable()
			end,
		},
		on_submit = function(item, idx)
			on_choice(item.value, idx)
		end,
		title = title,
	}):find()
end
