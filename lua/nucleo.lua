local api = vim.api

local M = {}

--- @private
M._rust = {
	Picker = true,
	FilePicker = true,
	CustomPicker = true,
	Previewer = true,
	LuaPicker = true,
}

function M.setup(...)
	require("nucleo.config").setup(...)
	api.nvim_create_user_command("Nucleo", function()
		M.find()
	end, {})
	api.nvim_create_user_command("CustomPicker", function()
		M.source_test()
	end, {})
	api.nvim_create_user_command("LuaPicker", function()
		M.lua_test()
	end, {})
end

function M.find(...)
	local Picker = require("nucleo.picker")

	Picker({
		on_submit = function(selection)
			local path = selection.value.path
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
				{
					display = "Felipe Handsome",
					selected = false,
					indices = {},
					value = {
						display = "Custom Lua",
						value = {
							line = 1,
							col = 12,
							wtf = "",
							hello = {},
						},
					},
				},
				-- { display = "Felipe Extra Handsome", value = { text = "Felipe is really extra handsome" } },
			},
		},
		on_submit = function(selection)
			local path = selection.value.path
			vim.cmd.drop(string.format("%s", vim.fn.fnameescape(path)))
		end,
	}):find(...)
end

function M.lua_test(...)
	local Picker = require("nucleo.picker")

	Picker({
		-- Builtin: source = "builtin.files",
		source = function()
			return vim.iter(vim.diagnostic.get(nil))
				:map(function(diagnostic)
					return {
						display = vim.split(diagnostic.message, "\n")[1],
						value = diagnostic,
					}
				end)
				:totable()
		end,
		on_submit = function(selection)
			local path = selection.value.path
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
