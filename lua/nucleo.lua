local M = {}

--- @private
M._rust = {
	files = true,
	fuzzy_match = true,
}

return setmetatable(M, {
	__index = function(t, key)
		if M._rust[key] then
			t[key] = require("nucleo_nvim")[key]
			return t[key]
		end
	end,
})
