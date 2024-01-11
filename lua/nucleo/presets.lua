local Layout = require("nui.layout")

local M = {}

function M.horizontal()
	return function(prompt, results, previewer)
		return Layout(
			{
				relative = "editor",
				position = "50%",
				size = {
					width = "80%",
					height = "80%",
				},
			},
			Layout.Box({
				Layout.Box({
					Layout.Box(prompt, { size = { width = "100%", height = 3 } }),
					Layout.Box(results, { grow = 1 }),
				}, { dir = "col", size = "50%" }),
				Layout.Box({
					Layout.Box(previewer, { size = "100%" }),
				}, { dir = "col", size = "50%" }),
			}, { dir = "row" })
		)
	end
end

function M.center() end

return M
