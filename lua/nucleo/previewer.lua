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
local function has_ts_parser(lang)
	return pcall(vim.treesitter.language.add, lang)
end
function Previewer:render(file)
	if self.winid then
		local height = vim.api.nvim_win_get_height(self.winid)
		local lines = preview_file(file, height)
		vim.api.nvim_buf_set_lines(self.bufnr, 0, -1, false, vim.split(lines, "\n"))
		vim.schedule(function()
			local ft = vim.filetype.match({ filename = file })
			if not ft or ft == "" then
				return
			end

			local lang = vim.treesitter.language.get_lang(ft)
			if lang and has_ts_parser(lang) then
				return vim.treesitter.start(self.bufnr, lang)
			end
			-- vim.api.nvim_buf_set_option(self.bufnr, "filetype", ft)
		end)
	end
end

return Previewer
