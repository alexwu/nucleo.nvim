local M = {}

---@param picker Picker
---@param results Results
function M.jump(picker, results)
	local has_flash, flash = pcall(require, "flash")
	if not has_flash then
		return
	end

	flash.jump({
		pattern = "^.",
		label = { after = { 0, 0 } },
		search = {
			mode = "search",
			multi_window = true,
			exclude = {
				function(win)
					return win ~= results.winid
				end,
			},
		},
		action = function(match)
			if results.sort_direction == "ascending" then
				picker:set_cursor(picker:window_height() - match.pos[1])
			else
				picker:set_cursor(match.pos[1] - 1)
			end
		end,
		highlight = { backdrop = false },
	})
end

return M
