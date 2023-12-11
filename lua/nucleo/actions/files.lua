local cmd = vim.cmd

local M = {}

---@enum Commands
local OPEN_CMD = {
	Edit = cmd.edit,
	Drop = cmd.drop,
	Vsplit = cmd.vsplit,
}

---@param	filename string
---@param command string
function M.open_file(filename, command)
	local open = OPEN_CMD[command] or OPEN_CMD.Drop

	local path = string.format("%s", vim.fn.fnameescape(filename))
	open(path)
end

return M
