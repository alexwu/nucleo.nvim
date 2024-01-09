local Popup = require("nui.popup")
local a = require("nucleo.async").a
local await_schedule = require("nucleo.async").scheduler
local api = vim.api

local ns_preview_match = api.nvim_create_namespace("nucleo/preview_match")

---@diagnostic disable-next-line: undefined-field
local Previewer = Popup:extend("Previewer")

local function highlight_match(bufnr, preview_options, offset)
	if not preview_options.line_end or not preview_options.col_end then
		return
	end

	local adjusted_line_start = math.max(preview_options.line_start - offset, 0)
	local adjusted_line_end = math.max(preview_options.line_end - offset, 0)

	api.nvim_buf_set_extmark(bufnr, ns_preview_match, adjusted_line_start, preview_options.col_start, {
		end_col = preview_options.col_end,
		end_row = adjusted_line_end,
		hl_group = "TelescopePreviewMatch",
	})
end

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

Previewer.render = a.void(function(self, entry)
	if not self.winid or not entry or not entry.value then
		return
	end

	local preview_options = entry.preview_options or {}
	if preview_options.kind == "skip" then
		self:clear()
		return
	end

	local start = preview_options.line_start or 0
	local ft = preview_options.file_extension

	local height = api.nvim_win_get_height(self.winid)
	local path
	if preview_options.bufnr and not preview_options.uri then
		local uri = vim.uri_from_bufnr(preview_options.bufnr)
		local fname = vim.uri_to_fname(uri)
		if fname then
			path = fname
		end
	elseif preview_options.uri then
		local fname = vim.uri_to_fname(preview_options.uri)
		if fname then
			path = fname
		end
	else
		path = preview_options.path or entry.value.path
	end

	if not path then
		return
	end

	local content, offset
	if preview_options.kind == "folder" then
		content = self.previewer:preview_folder(path)
		content = vim.iter(content)
			:map(function(line)
				return vim.fs.basename(line)
			end)
			:filter(function(line)
				return vim.fn.strlen(line) > 0
			end)
			:totable()
	elseif preview_options.kind == "diff" then
		content = self.previewer:preview_diff(path)
	else
		content, offset = unpack(self.previewer:preview_file(path, start, start + height))
	end
	api.nvim_buf_set_lines(self.bufnr, 0, -1, false, content)

	local line_count = api.nvim_buf_line_count(self.bufnr)
	if line_count == 0 then
		return
	end

	local name = vim.fs.basename(path)
	ft = vim.filetype.match({ filename = name, content = content })

	if not ft or ft == "" then
		return
	end

	await_schedule()
	-- vim.schedule(function()
	local lang = vim.treesitter.language.get_lang(ft)
	if lang and has_ts_parser(lang) then
		vim.treesitter.start(self.bufnr, lang)
	else
		vim.treesitter.stop(self.bufnr)
		vim.bo[self.bufnr].syntax = ft
	end

	if preview_options.kind == "diff" then
		vim.treesitter.start(self.bufnr, "diff")
		vim.bo[self.bufnr].syntax = "diff"
	end

	highlight_match(self.bufnr, preview_options, offset)
	-- end)
end)

return Previewer
