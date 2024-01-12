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
				Layout.Box(results, { size = "50%" }),
				Layout.Box(prompt, { size = { width = "100%", height = 3 } }),
				Layout.Box(previewer, { grow = 1 }),
			}, { dir = "col", size = "100%" })
		)
	end
end

return M
