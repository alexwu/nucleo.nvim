local M = {}

--- @private
M._rust = {
	files = true,
	fuzzy_match = true,
	fuzzy_match_with_scores = true,
	matches = true,
	set_picker_items = true,
	update_query = true,
	restart_picker = true,

	init_file_finder = true,
	fuzzy_file = true,
	init_picker = true,
	current_matches = true,
}

function M.setup()
	require("nucleo.pickers").setup()
end

return setmetatable(M, {
	__index = function(t, key)
		if M._rust[key] then
			t[key] = require("nucleo_nvim")[key]
			return t[key]
		end
	end,
})
