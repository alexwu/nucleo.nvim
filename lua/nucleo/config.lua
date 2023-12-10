---@type Nucleo.Config
local M = {}

---@class Nucleo.Config
local defaults = {
	sort_direction = "descending",
}

---@type Nucleo.Config
local options

---@param opts? Nucleo.Config
function M.setup(opts)
	opts = opts or {}

	options = M.get(opts)
end

---@param ... Nucleo.Config|nil
---@return Nucleo.Config
function M.get(...)
	if options == nil then
		M.setup()
	end

	return vim.tbl_get(options, ...)
end

return setmetatable(M, {
	__index = function(_, key)
		if options == nil then
			M.setup()
		end
		return options[key]
	end,
})
