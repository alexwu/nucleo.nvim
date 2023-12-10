local M = {}

---@param picker Nucleo.Picker
function M.close(picker)
	picker.layout:unmount()
end

---@param picker Nucleo.Picker
function M.move_cursor_up(picker)
	picker.picker:move_cursor_up()
end

---@param picker Nucleo.Picker
function M.move_cursor_down(picker)
	picker.picker:move_cursor_down()
end

---@param picker Nucleo.Picker
function M.move_to_top(picker)
	picker.picker:move_to_top()
end

---@param picker Nucleo.Picker
function M.move_to_bottom(picker)
	picker.picker:move_to_bottom()
end

---@param picker Nucleo.Picker
function M.scroll_up(picker)
	local delta = tonumber(vim.split(vim.opt.mousescroll:get()[1], ":")[2])
	picker.picker:move_cursor_up(delta)
end

---@param picker Nucleo.Picker
function M.scroll_down(picker)
	local delta = tonumber(vim.split(vim.opt.mousescroll:get()[1], ":")[2])
	picker.picker:move_cursor_down(delta)
end

---@param picker Nucleo.Picker
function M.multiselect(picker)
	local pos = picker.picker:get_cursor_pos()
	if pos then
		picker.picker:select(pos)
		picker.tx.send()
	end
end

return M
