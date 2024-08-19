local api = vim.api

local M = {}

--- @private
M._rust = {
	Picker = true,
	Previewer = true,
}

function M.setup(...)
	local ok, nucleo_rs = pcall(require, "nucleo_rs")
	if not ok then
		vim.notify("nucleo_rs executable missing, please build and try again", vim.log.levels.WARN)
		return
	end

	local config = require("nucleo.config")
	config.setup(...)

	nucleo_rs.setup(config.get("defaults"))

	if config.get("override_vim_select") then
		vim.ui.select = require("nucleo.vim").select
	end

	-- TODO: Should I move this lower?
	vim.g.nucleo_loaded = 1

	api.nvim_create_user_command("Find", function()
		M.find()
	end, { desc = "Find files" })

	api.nvim_create_user_command("Hunks", function()
		require("nucleo.sources.git").git_hunks()
	end, {})
	api.nvim_create_user_command("Select", function()
		M.select({
			{ ordinal = "foo", score = 0, kind = "lua", selected = false, indices = {}, value = { val = "foo" } },
			{ ordinal = "bar", score = 0, kind = "lua", selected = false, indices = {}, value = { val = "bar" } },
			{ ordinal = "baz", score = 0, kind = "lua", selected = false, indices = {}, value = { val = "baz" } },
		}, {}, function(item, idx) end)
	end, {})
end

function M.find(...)
	require("nucleo.sources").find_files(...)
end

return setmetatable(M, {
	__index = function(t, key)
		if M._rust[key] then
			t[key] = require("nucleo_rs")[key]
			return t[key]
		end
	end,
})
