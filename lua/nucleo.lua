local api = vim.api

local M = {}

--- @private
M._rust = {
	Picker = true,
	FilePicker = true,
	GitStatusPicker = true,
	Previewer = true,
}

function M.setup(...)
	require("nucleo.config").setup(...)
	api.nvim_create_user_command("Nucleo", function()
		M.find()
	end, {})
	api.nvim_create_user_command("GitStatusPicker", function()
		M.find_git({ cwd = vim.uv.cwd })
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

function M.find_git(...)
	local Picker = require("nucleo.picker")

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
return setmetatable(M, {
	__index = function(t, key)
		if M._rust[key] then
			t[key] = require("nucleo_rs")[key]
			return t[key]
		end
	end,
})
