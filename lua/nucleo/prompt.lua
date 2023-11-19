local Input = require("nui.input")
local Text = require("nui.text")
local api = vim.api

local ns_match_count = vim.api.nvim_create_namespace("nucleo_match_count")

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

	self.extmark_id = nil

	Prompt.super.init(self, popup_options, input_options)
end

function Prompt:render_match_count(total_matches, total_options)
	if not self.bufnr or not vim.api.nvim_buf_is_loaded(self.bufnr) then
		return
	end

	local match_count_str = string.format("%s / %s", total_matches, total_options)
	self.extmark_id = api.nvim_buf_set_extmark(self.bufnr, ns_match_count, 0, 0, {
		id = self.extmark_id,
		virt_text = { { match_count_str, "TelescopePromptCounter" } },
		virt_text_pos = "right_align",
	})
end

return Prompt
