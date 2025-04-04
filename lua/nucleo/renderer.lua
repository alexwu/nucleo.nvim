local Input = require("nui.input")
local Layout = require("nui.layout")
local Results = require("nucleo.results")

---@class Manager: NuiLayout
local Manager = Layout:extend("Manager")

function Manager:init(layout_options)
	local options = vim.tbl_deep_extend("force", layout_options or {}, {
		relative = "editor",
		position = "50%",
		size = {
			width = "50%",
			height = "80%",
		},
	})

	self.input = Input({}, {})
	self.results = Results()

	Manager.super.init(
		self,
		options,
		Layout.Box({
			Layout.Box(self.input, { size = {
				width = "100%",
				height = "3",
			} }),
			Layout.Box(self.results, { size = "100%" }),
		}, { dir = "col" })
	)
end

return Manager
