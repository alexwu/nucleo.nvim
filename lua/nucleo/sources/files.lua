local Picker = require("nucleo.picker")

local M = {}

function M.find_files(...)
	Picker({
		source = "builtin.files",
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
