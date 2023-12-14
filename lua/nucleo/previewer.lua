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

function Previewer:reset()
	self.previewer:reset()
end

function Previewer:clear()
	api.nvim_buf_set_lines(self.bufnr, 0, -1, false, {})
end

function Previewer:render(entry)
	if not self.winid or not entry or not entry.value then
		return
	end

	local preview_options = entry.preview_options or {}
	local start = preview_options.line_start or 0
	local height = api.nvim_win_get_height(self.winid)
	local path
	if preview_options.bufnr then
		local uri = vim.uri_from_bufnr(entry.preview_options.bufnr)
		local fname = vim.uri_to_fname(uri)
		if fname then
			path = fname
		end
	else
		path = entry.value.path
	end

	if not path then
		return
	end

	local content = self.previewer:preview_file(path, start, start + height)
	-- local content = vim.split(lines, "\n")
	api.nvim_buf_set_lines(self.bufnr, 0, -1, false, content)

	local line_count = api.nvim_buf_line_count(self.bufnr)
	if line_count == 0 then
		return
	end

	vim.schedule(function()
		local name = vim.fs.basename(path)
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

return Previewer
