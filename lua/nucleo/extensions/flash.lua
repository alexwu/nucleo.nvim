local M = {}

---@param picker Picker
---@param results_winid integer
function M.jump(picker, results_winid)
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
					return win ~= results_winid
				end,
			},
		},
		action = function(match)
			picker:set_cursor(match.pos[1] - 1)
		end,
		highlight = { backdrop = false },
	})
end

return M
