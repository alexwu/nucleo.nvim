local M = {}

--- @private
M._rust = {
	init_picker = true,
	Picker = true,
	preview_file = true,
}

function M.setup(opts)
	opts = vim.F.if_nil(opts, {})
	require("nucleo.pickers").setup()
end

function M.find()
	require("nucleo.pickers").find()
end

return setmetatable(M, {
	__index = function(t, key)
		if M._rust[key] then
			t[key] = require("nucleo_nvim")[key]
			return t[key]
		end
	end,
})
