local Input = require("nui.input")
local Text = require("nui.text")

---@class Prompt: NuiInput
local Prompt = Input:extend("Prompt")

function Prompt:init(opts)
	local popup_options = vim.tbl_deep_extend("force", opts.popup_options or {}, {
		position = "50%",
		size = {
			width = 20,
			height = 1,
		},
		border = {
			style = "rounded",
			text = {
				top = "",
				top_align = "center",
			},
		},
		buf_options = {
			filetype = "nucleo",
		},
		win_options = {
			winhighlight = "Normal:Normal,FloatBorder:FloatBorder",
		},
	})
	local input_options = vim.tbl_deep_extend("force", opts.input_options or {}, {
		prompt = Text(" ", "TelescopePromptPrefix"),
		default_value = "",
	})

	Prompt.super.init(self, popup_options, input_options)
end

return Prompt
