local Input = require("nui.input")

local Prompt = Input:extend("Prompt")

function Prompt:init(input_options)
	local options = vim.tbl_deep_extend("force", input_options or {}, {
		border = "rounded",
		focusable = true,
		position = { row = 0, col = "100%" },
		size = { width = 10, height = 1 },
		win_options = {
			winhighlight = "Normal:Normal,FloatBorder:Normal",
		},
		line = 0,
		options = {},
	})

	Prompt.super.init(self, options, {})
end

return Prompt
