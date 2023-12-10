local Highlighter = require("nucleo.highlighter")
local debounce = require("nucleo.debounce").debounce_trailing
local Layout = require("nui.layout")
local log = require("nucleo.log")
local channel = require("plenary.async.control").channel
local Previewer = require("nucleo.previewer")
local Prompt = require("nucleo.prompt")
local event = require("nui.utils.autocmd").event
local Results = require("nucleo.results")
local a = require("plenary.async")
local await_schedule = a.util.scheduler
local extensions = require("nucleo.extensions")
local nu = require("nucleo_rs")

local api = vim.api

---@class PickerBackend
---@field drain_channel fun(self: PickerBackend)
---@field force_rerender fun(self: PickerBackend)
---@field get_cursor_pos fun(self: PickerBackend): integer|nil
---@field get_selection fun(self: PickerBackend): PickerEntry
---@field selections fun(self: PickerBackend): PickerEntry[]
---@field selection_indices fun(self: PickerBackend): integer[]
---@field move_cursor_down fun(self: PickerBackend, delta?: integer)
---@field move_cursor_up fun(self: PickerBackend, delta?: integer)
---@field move_to_bottom fun(self: PickerBackend)
---@field move_to_top fun(self: PickerBackend)
---@field populate_files fun(self: PickerBackend)
---@field restart fun(self: PickerBackend)
---@field select fun(self: PickerBackend, pos: integer)
---@field set_cursor fun(self: PickerBackend, pos: integer)
---@field should_rerender fun(self: PickerBackend): boolean
---@field sort_direction fun(self: PickerBackend): "descending"|"ascending"
---@field tick fun(self: PickerBackend, timeout: integer): PickerStatus
---@field total_items fun(self: PickerBackend): integer
---@field total_matches fun(self: PickerBackend): integer
---@field update_config fun(self: PickerBackend, config: Nucleo.FilePicker.Config)
---@field update_cwd fun(self: PickerBackend, cwd: string)
---@field update_query fun(self: PickerBackend, query: string)
---@field update_window fun(self: PickerBackend, height: integer)
---@field window_height fun(self: PickerBackend): integer

---@class Nucleo.Picker: Object
local Picker = require("plenary.class"):extend()

function Picker:new(opts)
	opts = opts or {}
	vim.validate({
		on_submit = { opts.on_submit, "function" },
	})

	self.original_winid = api.nvim_get_current_win()
	self.original_cursor = api.nvim_win_get_cursor(self.original_winid)
	self.timer = vim.uv.new_timer()
	---@type Sender, Receiver
	self.tx, self.rx = channel.counter()
	self.picker = nu.Picker(opts)
	self.results = Results()
	self.previewer = Previewer()
	self.highlighter = Highlighter({
		picker = self.picker,
		results = self.results,
	})

	self.prompt = Prompt({
		picker = self.picker,
		input_options = {
			on_close = function()
				self:shutdown_timer()
				self:close()
			end,
			on_submit = function()
				self:shutdown_timer()

				if self.picker:total_matches() == 0 then
					vim.notify("There's nothing to select", vim.log.levels.WARN)

					self:close()
				else
					self:submit()
				end
			end,
			on_change = function(val)
				self:process_input(val)
			end,
		},
	})

	self._on_close = opts.on_close
	self._on_submit = opts.on_submit

	self.layout = Layout(
		{
			relative = "editor",
			position = "50%",
			size = {
				width = "80%",
				height = "80%",
			},
		},
		Layout.Box({
			Layout.Box(self.prompt, { size = { width = "100%", height = "3" } }),
			Layout.Box({
				Layout.Box(self.results, { size = "40%" }),
				Layout.Box(self.previewer, { size = "60%" }),
			}, { dir = "row", size = "100%" }),
		}, { dir = "col" })
	)

	self.prompt:map("n", "<Esc>", function()
		self.prompt:unmount()
	end, { noremap = true })

	self.prompt:map("i", "<Esc>", function()
		self.prompt:unmount()
	end, { noremap = true })

	self.prompt:map("i", { "<C-n>", "<Down>" }, function()
		self.picker:move_cursor_down()
		self.tx.send()
	end, { noremap = true })

	self.prompt:map("i", { "<C-p>", "<Up>" }, function()
		self.picker:move_cursor_up()
		self.tx.send()
	end, { noremap = true })

	self.prompt:map("i", { "<ScrollWheelUp>" }, function()
		local delta = tonumber(vim.split(vim.opt.mousescroll:get()[1], ":")[2])
		self.picker:move_cursor_up(delta)
		self.tx.send()
	end, { noremap = true })

	self.prompt:map("i", { "<ScrollWheelDown>" }, function()
		local delta = tonumber(vim.split(vim.opt.mousescroll:get()[1], ":")[2])
		self.picker:move_cursor_down(delta)
		self.tx.send()
	end, { noremap = true })

	self.prompt:map("i", { "<Tab>" }, function()
		local pos = self.picker:get_cursor_pos()
		if pos then
			self.picker:select(pos)
			self.tx.send()
		end
	end, { noremap = true })

	self.prompt:map("i", { "<C-b>" }, function()
		self.picker:move_to_top()
		self.tx.send()
	end, { noremap = true })

	self.prompt:map("i", { "<C-f>" }, function()
		self.picker:move_to_bottom()
		self.tx.send()
	end, { noremap = true })

	self.prompt:map("i", { "<C-r>" }, function()
		self.picker:tick(10)
		self.picker:force_rerender()
		self.tx.send()
	end, { noremap = true })

	self.prompt:map("i", { "<C-s>" }, function()
		extensions.flash.jump(self.picker, self.results)
		self.tx.send()
	end, { noremap = true })

	self.results:on(event.BufWinEnter, function()
		local height = math.max(api.nvim_win_get_height(self.results.winid), 10)

		self.picker:update_window(height)
	end)

	self.prompt:on("VimResized", function()
		local height = math.max(api.nvim_win_get_height(self.results.winid), 10)

		self.picker:update_window(height)
	end)

	self.prompt:on(event.BufLeave, function()
		self.layout:unmount()
	end)
end

---@param opts Nucleo.FilePicker.Config
function Picker:find(opts)
	opts = opts or {}
	self.picker:update_config(opts)
	self.picker:populate_files()
	self.picker:tick(10)

	self.picker:force_rerender()
	self.tx.send()

	self:render()
end

function Picker:submit()
	self:reset_cursor()

	local selection = self.picker:get_selection()
	if self._on_submit then
		self._on_submit(selection)
	end

	self.prompt:stop()
	-- TODO: Figure out what to actually do here
	self.picker:update_query("")
	self.picker:restart()
end

function Picker:close()
	self:reset_cursor()

	if self._on_close then
		self._on_close()
	end

	self.prompt:stop()
	self.picker:update_query("")
	self.picker:restart()
end

---@param self Nucleo.Picker
---@param val string
Picker.process_input = debounce(function(self, val)
	self.picker:update_query(val)
	-- self.picker:force_rerender()
	log.info("Updated input: " .. val)

	-- self.set_interval(10, self.check_for_updates)

	self.tx.send()
end, 50)

function Picker:shutdown_timer()
	if not self.timer:is_closing() then
		self.timer:stop()
		self.timer:close()
	end
end

function Picker:update_preview()
	if self.picker:total_matches() > 0 then
		self.previewer:render(self.picker:get_selection().path)
	else
		self.previewer:clear()
	end
end

function Picker:highlight_selection()
	if self.picker:total_matches() > 0 then
		self.highlighter:highlight_selection()
	end
end

function Picker:reset_cursor()
	if self.original_winid then
		api.nvim_set_current_win(self.original_winid)
	end
end

Picker.check_for_updates = vim.schedule_wrap(a.void(function(self)
	if not self.results or not self.picker then
		return
	end

	log.info("Checking for updates...")
	if not self.results.bufnr or not api.nvim_buf_is_loaded(self.results.bufnr) then
		self:shutdown_timer()
	end

	local status = self.picker:tick(10)
	if status.changed or status.running or self.picker:should_rerender() then
		self.picker:force_rerender()
		self.tx.send()
	end

	if not (status.running or status.changed or self.picker:should_rerender()) and self.picker:total_items() > 0 then
		self:shutdown_timer()
	end
end))

function Picker:render()
	self.layout:show()

	local main_loop = a.void(function()
		log.info("Starting main loop...")

		-- self.set_interval(10, self.check_for_updates)

		await_schedule()

		while true do
			log.info("Looping...")
			self.rx.last()
			await_schedule()

			if not self.results.bufnr or not api.nvim_buf_is_loaded(self.results.bufnr) or not self.results.winid then
				return
			end

			self:highlight_selection()
			self:update_preview()

			local status = self.picker:tick(10)
			if self.picker:should_rerender() or status.changed then
				log.info("trying to render in the main loop")
				log.info("Rendering with total matches: ", self.picker:total_matches())
				self.picker:drain_channel()
				self.results:render_entries(self.picker)
			end

			self:highlight_selection()
			self:update_preview()
		end
	end)

	self.prompt:update(100)
	self.tx.send()
	main_loop()
end

return Picker
