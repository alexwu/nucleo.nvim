local Popup = require("nui.popup")
local preview_file = require("nucleo").preview_file

local Previewer = Popup:extend("Previewer")

function Previewer:init(popup_options)
	local options = vim.tbl_deep_extend("force", popup_options or {}, {
		border = "rounded",
		focusable = false,
		-- position = { row = 0, col = "100%" },
		-- size = { width = 10, height = 1 },
		win_options = {
			winhighlight = "Normal:Normal,FloatBorder:FloatBorder",
		},
		options = {},
	})

	Previewer.super.init(self, options)
end

function Previewer:render(file)
	if self.winid then
		local height = vim.api.nvim_win_get_height(self.winid)
		local lines = preview_file(file, height)
		vim.api.nvim_buf_set_lines(self.bufnr, 0, -1, false, vim.split(lines, "\n"))
	end
end

return Previewer
