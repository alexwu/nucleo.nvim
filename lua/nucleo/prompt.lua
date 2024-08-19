local Input = require("nui.input")
local Text = require("nui.text")
local a = require("nucleo.async").a
local log = require("nucleo.log")
local channel = require("nucleo.async").channel
local scheduler_if_buf_valid = require("nucleo.async").scheduler_if_buf_valid

local api = vim.api

local ns_match_count = vim.api.nvim_create_namespace("nucleo_match_count")

---@class Nucleo.Prompt: NuiInput
---@field super NuiInput
---@diagnostic disable-next-line: undefined-field
local Prompt = Input:extend("Prompt")

---@class PromptConfig
---@field popup_options nui_popup_options
---@field input_options nui_input_options
---@field picker PickerBackend
---@field title? string

---@param opts PromptConfig
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
				top = opts.title or "",
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

	self.timer = vim.uv.new_timer()
	self.picker = opts.picker
	self.extmark_id = nil
	self.tx, self.rx = channel.counter()

	Prompt.super.init(self, popup_options, input_options)
end

---@param interval integer
function Prompt:update(interval)
	self.timer:start(
		interval,
		interval,
		a.void(function()
			local match_count = self.picker:total_matches()
			local item_count = self.picker:total_items()

			a.run(function()
				self.picker:tick(10)
			end, function()
				log.info("Rendering match count...")
				self:render_match_count(match_count, item_count)
			end)
		end)
	)
end

function Prompt:stop()
	self.extmark_id = nil
	if self.timer:is_closing() then
		return
	end

	self.timer:stop()
	self.timer:close()
end

---@param total_matches number
---@param total_options number
function Prompt:render_match_count(total_matches, total_options)
	scheduler_if_buf_valid(self.bufnr, function()
		local match_count_str = string.format("%s / %s", total_matches, total_options)
		self.extmark_id = api.nvim_buf_set_extmark(self.bufnr, ns_match_count, 0, 0, {
			id = self.extmark_id,
			virt_text = { { match_count_str, "TelescopePromptCounter" } },
			virt_text_pos = "right_align",
		})
	end)
end

return Prompt
