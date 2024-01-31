local api = vim.api

local M = {}

--- @private
M._rust = {
	FilePicker = true,
	GitStatusPicker = true,
	LuaPicker = true,
	Previewer = true,
}

function M.setup(...)
	local config = require("nucleo.config")
	config.setup(...)

	require("nucleo_rs").setup(config.get("defaults"))

	api.nvim_create_user_command("Nucleo", function()
		M.find()
	end, {})

	api.nvim_create_user_command("Hunks", function()
		require("nucleo.sources.git").git_hunks()
	end, {})
end

function M.find(...)
	local Picker = require("nucleo.picker")

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

return setmetatable(M, {
	__index = function(t, key)
		if M._rust[key] then
			t[key] = require("nucleo_rs")[key]
			return t[key]
		end
	end,
})
