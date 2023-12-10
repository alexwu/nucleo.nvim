local M = {}

---@param picker Nucleo.Picker
function M.jump(picker)
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
					return win ~= picker.results.winid
				end,
			},
		},
		action = function(match)
			if picker.picker:sort_direction() == "ascending" then
				picker.picker:set_cursor(picker.picker:window_height() - match.pos[1])
			else
				picker.picker:set_cursor(match.pos[1] - 1)
			end
		end,
		highlight = { backdrop = false },
	})
end

return M
