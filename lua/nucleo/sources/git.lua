local Picker = require("nucleo.picker")

local M = {}

function M.git_status(...)
	Picker({
		source = "builtin.git_status",
		cwd = vim.uv.cwd,
		on_submit = function(selection)
			local path = selection.value.path
			if path then
				vim.cmd.drop(string.format("%s", vim.fn.fnameescape(path)))
			else
				vim.print(selection.value)
			end
		end,
	}):find(...)
end

return M
