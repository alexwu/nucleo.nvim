local Highlighter = require("nucleo.highlighter")
local debounce = require("nucleo.debounce").debounce_trailing
local Layout = require("nui.layout")
local log = require("nucleo.log")
local channel = require("nucleo.async").channel
local Previewer = require("nucleo.previewer")
local Prompt = require("nucleo.prompt")
local config = require("nucleo.config")
local event = require("nui.utils.autocmd").event
local Results = require("nucleo.results")
local a = require("nucleo.async").a
local await_schedule = require("nucleo.async").scheduler
local nu = require("nucleo_rs")

local api = vim.api

---@class Receiver
---@field recv fun()
---@field last fun()

---@class Sender
---@field send fun()

---@class PickerStatus
---@field running boolean
---@field changed boolean

---@class CustomEntry
---@field display string
---@field value string
---@field selected boolean

---@class PickerBackend
---@field drain_channel fun(self: PickerBackend)
---@field force_rerender fun(self: PickerBackend)
---@field get_cursor_pos fun(self: PickerBackend): integer|nil
---@field get_selection fun(self: PickerBackend): Nucleo.Picker.Entry
---@field move_cursor_down fun(self: PickerBackend, delta?: integer)
---@field move_cursor_up fun(self: PickerBackend, delta?: integer)
---@field move_to_bottom fun(self: PickerBackend)
---@field move_to_top fun(self: PickerBackend)
---@field populate_files fun(self: PickerBackend)
---@field populate fun(self: PickerBackend, entries: CustomEntry[])
---@field restart fun(self: PickerBackend)
---@field multiselect fun(self: PickerBackend, pos: integer)
---@field toggle_selection fun(self: PickerBackend, pos: integer)
---@field selection_indices fun(self: PickerBackend): integer[]
---@field selections fun(self: PickerBackend): Nucleo.Picker.Entry[]
---@field set_cursor fun(self: PickerBackend, pos: integer)
---@field should_rerender fun(self: PickerBackend): boolean
---@field sort_direction fun(self: PickerBackend): "descending"|"ascending"
---@field tick fun(self: PickerBackend, timeout: integer): PickerStatus
---@field total_items fun(self: PickerBackend): integer
---@field total_matches fun(self: PickerBackend): integer
---@field update_config fun(self: PickerBackend, config: Nucleo.FilePicker.Config)
---@field update_query fun(self: PickerBackend, query: string)
---@field update_window fun(self: PickerBackend, height: integer)
---@field window_height fun(self: PickerBackend): integer

---@class Nucleo.FilePicker.Config
---@field git_ignore boolean
---@field cwd fun()|string
---@field sort_direction "descending"|"ascending"
---@field ignore boolean

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
	---@type PickerBackend
	if not opts.source then
		self.picker = nu.Picker(opts)
	else
		self.picker = nu.CustomPicker(opts.source)
	end

	self.results = Results()
	self.previewer = Previewer()
	self.highlighter = Highlighter({
		picker = self.picker,
		results = self.results,
	})
	self.source = opts.source

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

	local default_mappings = config.get("mappings")

	vim.iter(default_mappings):each(function(mode, mappings)
		vim.iter(mappings):each(function(key, mapping)
			self:apply_mapping(mode, key, mapping)
		end)
	end)

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
---@param opts? Nucleo.FilePicker.Config
---@return Nucleo.FilePicker.Config
local function override(opts)
	opts = opts or {}

	local configs = { config.get("defaults"), config.get("sources", "files"), opts }

	return vim.tbl_deep_extend("force", unpack(configs))
end

---@param mode 'i'|'n'
---@param key string
---@param mapping Nucleo.Keymap
function Picker:apply_mapping(mode, key, mapping)
	vim.validate({
		callback = { mapping[1], "f" },
	})

	local opts = vim.tbl_extend("force", mapping.opts, { buffer = self.prompt.bufnr })

	vim.keymap.set(mode, key, function()
		mapping[1](self)
		self.tx:send()
	end, opts)
end

---@param opts Nucleo.FilePicker.Config
function Picker:find(opts)
	local options = override(opts)

	self.picker:update_config(options)
	if not self.source then
		self.picker:populate_files()
	else
		vim.print(self.source)
		self.picker:populate(self.source.results)
	end

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

	-- self:check_for_updates(10)

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

---@param self Nucleo.Picker
---@param interval integer
Picker.check_for_updates = function(self, interval)
	if self.timer:is_closing() then
		return
	elseif self.timer:is_active() and interval ~= self.timer:get_repeat() then
		self.timer:set_repeat(interval)
		return
	elseif self.timer:is_active() then
		return
	end

	self.timer:start(
		interval,
		interval,
		a.void(function()
			if not self.results or not self.picker then
				self:shutdown_timer()
				return
			end

			await_schedule()

			log.info("Checking for updates...")
			if not self.results.bufnr or not api.nvim_buf_is_loaded(self.results.bufnr) then
				self:shutdown_timer()
				return
			end

			-- TODO: How do i stop from re-rendering when there's no real changes to the buffer?
			local status = self.picker:tick(10)
			if
				(self.picker:total_items() < self.picker:window_height() and (status.changed or status.running))
				or self.picker:should_rerender()
			then
				self.picker:force_rerender()
				self.tx.send()
			end
		end)
	)
end

function Picker:render()
	self.layout:show()

	local main_loop = a.void(function()
		log.info("Starting main loop...")

		self:check_for_updates(100)

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
