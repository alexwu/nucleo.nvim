local Picker = require("nucleo.picker")
local presets = require("nucleo.presets")

local M = {}

function M.diagnostics(...)
	Picker({
		source = {
			name = "builtin.diagnostics",
			config = {
				scope = "workspace",
				sort_direction = "ascending",
			},
			finder = function(bufnr)
				return vim.diagnostic.get(bufnr, {})
			end,
		},
		layout = presets.center(),
		on_submit = function(selection)
			local bufnr = selection.value.bufnr
			local lnum = selection.value.lnum
			local col = selection.value.col
			if bufnr then
				vim.cmd.buffer(bufnr)
				vim.api.nvim_win_set_cursor(0, { lnum, col })
			else
				vim.print(selection.value)
			end
		end,
	}):find(...)
end

return M
