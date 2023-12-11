local files = require("nucleo.actions.files")

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
		picker.picker:multiselect(pos)
		picker.picker:force_rerender()
	end
end

---@param picker Nucleo.Picker
function M.toggle_selection(picker)
	local pos = picker.picker:get_cursor_pos()
	if pos then
		picker.picker:toggle_selection(pos)
		picker.picker:force_rerender()
	end
end

---@param picker Nucleo.Picker
function M.force_refresh(picker)
	picker.picker:tick(10)
	picker.picker:force_rerender()
end

---@param picker Nucleo.Picker
function M.open_in_vsplit(picker)
	if picker.picker:total_matches() == 0 then
		vim.notify("There's nothing to select", vim.log.levels.WARN)
	else
		picker:reset_cursor()

		local selection = picker.picker:get_selection()
		files.open_file(selection.path, "Vsplit")

		picker.prompt:stop()
		picker.picker:update_query("")
		picker.picker:restart()
	end
end

return M
