local Picker = require("nucleo.picker")
local presets = require("nucleo.presets")

---@param items any[]
---@param opts table
---@param on_choice fun(item: any?, idx: integer?)
return function(items, opts, on_choice)
	-- local format_item = opts.format_item or function(item)
	-- 	return item
	-- end

	local title = opts.prompt or ""

	Picker({
		source = {
			name = "custom.ui.select",
			config = {},
			results = {},
			format_item = opts.format_item or function(item)
				return item
			end,
			finder = function()
				return items
			end,
		},
		layout = presets.dropdown(),
		on_submit = function(item, idx)
			on_choice(item, idx)
		end,
		title = title,
	}):find()
end
