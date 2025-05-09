local M = {}

--- @private
M._rust = {
	Picker = true,
	Previewer = true,
}

function M.setup(...)
	require("nucleo.pickers").setup(...)
end

function M.find(...)
	require("nucleo.pickers").find(...)
end

return setmetatable(M, {
	__index = function(t, key)
		if M._rust[key] then
			t[key] = require("nucleo_rs")[key]
			return t[key]
		end
	end,
})
