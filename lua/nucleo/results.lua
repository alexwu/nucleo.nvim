local Popup = require("nui.popup")

local Results = Popup:extend("Results")

function Results:init(popup_options)
	local options = vim.tbl_deep_extend("force", popup_options or {}, {
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

	self.selection_index = 1

	Results.super.init(self, options)
end

return Results
