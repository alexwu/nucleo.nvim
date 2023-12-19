local cmd = vim.cmd

local M = {}

---@enum Commands
local OPEN_CMD = {
	edit = "buffer",
	new = "sbuffer",
	vsplit = "vert sbuffer",
	tab = "tab edit",
	drop = "drop",
}

---@param	filename string
---@param command string
function M.open_file(filename, command)
	local open_cmd = OPEN_CMD[command] or "buffer"
	local uri = vim.uri_from_fname(filename)
	local bufnr = vim.uri_to_bufnr(uri)

	cmd(string.format("%s %d", open_cmd, bufnr))
end

---@param command Commands
function M.select_file(command)
	---@param picker Nucleo.Picker
	return function(picker)
		if picker.picker:total_matches() == 0 then
			vim.notify("There's nothing to select", vim.log.levels.WARN)
		else
			picker:reset_cursor()

			local selection = picker.picker:get_selection()
			M.open_file(selection.value.path, command)

			picker.prompt:stop()
			picker.picker:update_query("")
			picker.picker:restart()
		end
	end
end

return M
