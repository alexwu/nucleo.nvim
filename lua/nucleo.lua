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
			if path then
				vim.cmd.drop(string.format("%s", vim.fn.fnameescape(path)))
			else
				vim.print(selection.value)
			end
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
				{ display = "Felipe Very Handsome", value = "Felipe Extra Handsome" },
			},
		},
		on_submit = function(selection)
			vim.print(selection)
		end,
	}):find(...)
end

function M.lua_test(...)
	local Picker = require("nucleo.picker")

	Picker({
		-- Builtin: source = "builtin.files",
		source = function()
			-- return vim.iter(pairs(vim.diagnostic.get(nil)))
			-- 	:map(function(diagnostic)
			-- 		local message = vim.split(diagnostic.message, "\n")[1]
			--
			-- 		local display = table.concat({ diagnostic.code, message }, " ")
			-- 		return {
			-- 			display = display,
			-- 			value = diagnostic,
			-- 		}
			-- 	end):totable()
			return vim.diagnostic.get(nil)
		end,
		on_submit = function(selection)
			vim.print(selection)
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
