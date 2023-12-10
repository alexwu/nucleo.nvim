local a = require("plenary.async")
local api = vim.api

local M = {}

---@type function:qa

M.scheduler = a.util.scheduler

--- @param buf? integer
--- @param cb function
M.scheduler_if_buf_valid = a.wrap(function(buf, cb)
	vim.schedule(function()
		if buf and api.nvim_buf_is_loaded(buf) then
			cb()
		end
	end)
end, 2)

return M
