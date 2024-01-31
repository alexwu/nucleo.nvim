local M = {}

function M.preview(previewer, entry)
	local preview_options = entry.preview_options
	local path = preview_options.path

	return previewer:preview_diff(path)
end

return M
