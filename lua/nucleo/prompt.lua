local Input = require("nui.input")
local Text = require("nui.text")
local a = require("plenary.async")
local log = require("nucleo.log")
local await_schedule = a.util.scheduler
local channel = require("plenary.async.control").channel

local api = vim.api

local ns_match_count = vim.api.nvim_create_namespace("nucleo_match_count")

---@class Prompt: NuiInput
local Prompt = Input:extend("Prompt")

---@class PromptConfig
---@field popup_options nui_popup_options
---@field input_options nui_input_options
---@field picker Picker

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
			await_schedule()
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
	if not self.timer:is_closing() then
		return
	end

	self.timer:stop()
	self.timer:close()
end

---@param total_matches number
---@param total_options number
function Prompt:render_match_count(total_matches, total_options)
	if not self.bufnr or not api.nvim_buf_is_loaded(self.bufnr) then
		return
	end

	local match_count_str = string.format("%s / %s", total_matches, total_options)
	self.extmark_id = api.nvim_buf_set_extmark(self.bufnr, ns_match_count, 0, 0, {
		id = self.extmark_id,
		virt_text = { { match_count_str, "TelescopePromptCounter" } },
		virt_text_pos = "right_align",
	})
end

---@param self Prompt
local render = a.void(function(self)
	while true do
		self.rx.last()
		await_schedule()

		if not self.bufnr or not api.nvim_buf_is_loaded(self.bufnr) then
			return
		end

		local match_count = self.picker:total_matches()
		local item_count = self.picker:total_items()

		self:render_match_count(match_count, item_count)
	end
end)

function Prompt:render()
	render(self)
end

return Prompt
