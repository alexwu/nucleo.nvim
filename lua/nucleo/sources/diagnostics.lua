local Picker = require("nucleo.picker")
local M = {}

function M.diagnostics(...)
	Picker({
		source = {
			name = "builtin.diagnostics",
			config = {
				scope = "workspace",
			},
			finder = function(bufnr)
				return vim.diagnostic.get(bufnr, {})
			end,
		},
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
