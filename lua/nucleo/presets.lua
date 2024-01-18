local Layout = require("nui.layout")

local M = {}

function M.horizontal()
	return function(prompt, results, previewer)
		return Layout(
			{
				relative = "editor",
				position = "50%",
				size = {
					width = "90%",
					height = "90%",
				},
			},
			Layout.Box({
				Layout.Box({
					Layout.Box(prompt, { size = 3 }),
					Layout.Box(results, { grow = 1 }),
				}, { dir = "col", grow = 1 }),
				Layout.Box({
					Layout.Box(previewer, { grow = 1 }),
				}, { dir = "col", grow = 1 }),
			}, { dir = "row" })
		)
	end
end

function M.center()
	return function(prompt, results, previewer)
		return Layout(
			{
				relative = "editor",
				position = "50%",
				size = {
					width = "50%",
					height = "90%",
				},
			},
			Layout.Box({
				Layout.Box(results, { grow = 1 }),
				Layout.Box(prompt, { size = 3 }),
				Layout.Box(previewer, { grow = 1 }),
			}, { dir = "col", size = "100%" })
		)
	end
end

---@class LayoutOptions
---@field relative "editor"|"cursor"|"mouse"
---@field position? (string|number|{ row: string|number, col: string|number })
---@field size? (string|number|{ width: string|number, height: string|number })

---@param opts? LayoutOptions
function M.make_layout(opts)
	opts = opts or {}

	local relative = opts.relative or "editor"
	local position = opts.position or "50%"
	local size = opts.size or { width = "50%", height = "90%" }

	return function(prompt, results, previewer)
		return Layout(
			{
				---@diagnostic disable-next-line: assign-type-mismatch
				relative = relative,
				position = position,
				size = size,
			},
			Layout.Box({
				Layout.Box(results, { size = "50%" }),
				Layout.Box(prompt, { size = { width = "100%", height = 3 } }),
				Layout.Box(previewer, { grow = 1 }),
			}, { dir = "col", size = "100%" })
		)
	end
end

return M
