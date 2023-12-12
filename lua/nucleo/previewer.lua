local Popup = require("nui.popup")
local api = vim.api

local Previewer = Popup:extend("Previewer")

function Previewer:init(popup_options)
	local options = vim.tbl_deep_extend("force", popup_options or {}, {
		border = "rounded",
		focusable = false,
		style = "minimal",
		win_options = {
			winhighlight = "Normal:Normal,FloatBorder:FloatBorder",
		},
		options = {},
	})

	self.previewer = require("nucleo_rs").Previewer()

	Previewer.super.init(self, options)
end

local function has_ts_parser(lang)
	return pcall(vim.treesitter.language.add, lang)
end

function Previewer:clear()
	api.nvim_buf_set_lines(self.bufnr, 0, -1, false, {})
end

function Previewer:render(file)
	if self.winid and file then
		local height = api.nvim_win_get_height(self.winid)
		local lines = self.previewer:preview_file(file, 0, height)
		local content = vim.split(lines, "\n")
		api.nvim_buf_set_lines(self.bufnr, 0, -1, false, content)

		local line_count = api.nvim_buf_line_count(self.bufnr)
		if line_count == 0 then
			return
		end

		vim.schedule(function()
			local name = vim.fs.basename(file)
			local ft = vim.filetype.match({ filename = name, content = content })
			if not ft or ft == "" then
				return
			end

			local lang = vim.treesitter.language.get_lang(ft)
			if lang and has_ts_parser(lang) then
				return vim.treesitter.start(self.bufnr, lang)
			end
		end)
	end
end

return Previewer
