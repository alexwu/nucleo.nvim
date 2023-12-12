local api = vim.api

local M = {}

--- @private
M._rust = {
	Picker = true,
	CustomPicker = true,
	Previewer = true,
}

function M.setup(...)
	require("nucleo.config").setup(...)
	api.nvim_create_user_command("Nucleo", function()
		M.find()
	end, {})
	api.nvim_create_user_command("CustomPicker", function()
		M.source_test()
	end, {})
end

function M.find(...)
	local Picker = require("nucleo.picker")

	Picker({
		on_submit = function(selection)
			local path = selection.path
			vim.cmd.drop(string.format("%s", vim.fn.fnameescape(path)))
		end,
	}):find(...)
end

function M.source_test(...)
	local Picker = require("nucleo.picker")

	Picker({
		-- Builtin: source = "builtin.files",
		source = {
			name = "Custom List",
			results = {
				{ display = "Felipe Handsome", value = "Felipe is really handsome" },
				{ display = "Felipe Extra Handsome", value = "Felipe is really extra handsome" },
			},
		},
		on_submit = function(selection)
			local path = selection.path
			vim.cmd.drop(string.format("%s", vim.fn.fnameescape(path)))
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
