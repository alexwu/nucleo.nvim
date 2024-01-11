local Picker = require("nucleo.picker")
local presets = require("nucleo.presets")

local M = {}

function M.find_files(...)
	Picker({
		source = {
			name = "builtin.files",
		},
		layout = presets.horizontal(),
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
