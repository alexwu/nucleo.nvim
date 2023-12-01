local Layout = require("nui.layout")
local event = require("nui.utils.autocmd").event
local Prompt = require("nucleo.prompt")
local Results = require("nucleo.results")
local a = require("plenary.async")
local await_schedule = a.util.scheduler
local channel = require("plenary.async.control").channel
local log = require("nucleo.log")
local nu = require("nucleo")
local debounce = require("nucleo.debounce").debounce_trailing
local Highlighter = require("nucleo.highlighter")
local Previewer = require("nucleo.previewer")
local api = vim.api

local M = {}

---@class PickerEntry
---@field path string
---@field file_type string

---@class PickerStatus
---@field running boolean
---@field changed boolean

---@class Picker
---@field update_query fun(self: Picker, query: string)
---@field update_cwd fun(self: Picker, cwd: string)
---@field update_window fun(self: Picker, height: integer)
---@field populate_files fun(self: Picker)
---@field restart fun(self: Picker)
---@field tick fun(self: Picker, timeout: integer): PickerStatus
---@field total_matches fun(self: Picker): integer
---@field total_items fun(self: Picker): integer
---@field should_rerender fun(self: Picker): boolean
---@field force_rerender fun(self: Picker)
---@field drain_channel fun(self: Picker)
---@field move_cursor_up fun(self: Picker)
---@field move_cursor_down fun(self: Picker)
---@field get_selection fun(self: Picker): PickerEntry
---@field get_cursor_pos fun(self: Picker): integer|nil

---@type Picker|nil
M.picker = nil
M.results = nil
M.highlighter = nil
M.original_cursor = nil
M.tx = nil

M.render_matches = function()
	M.results:render_entries(M.picker)
end

function M.queue_rerender()
	if M.tx then
		M.tx.send()
	end
end

---@param val string
M.process_input = debounce(function(val)
	M.picker:update_query(val)
	-- M.picker:force_rerender()
	log.info("Updated input: " .. val)

	M.set_interval(10, M.check_for_updates)

	-- if M.tx then
	-- 	M.tx.send()
	-- end
	M.queue_rerender()
end, 50)

---@param opts? PickerOptions
M.initialize = function(opts)
	opts = opts or { cwd = vim.uv.cwd() }
	M.main_timer = vim.uv.new_timer()
	if not M.picker then
		M.picker = nu.Picker(opts)
		M.picker:force_rerender()
	else
		if opts.cwd then
			M.picker:update_cwd(opts.cwd)
		end
		M.picker:populate_files()
		M.picker:tick(10)
		M.picker:force_rerender()

		M.queue_rerender()
	end
end

---@param interval integer
---@param callback function
function M.set_interval(interval, callback)
	if not M.main_timer then
		M.main_timer = vim.uv.new_timer()
	elseif M.main_timer:is_closing() then
		return
	elseif M.main_timer:is_active() and interval ~= M.main_timer:get_repeat() then
		M.main_timer:set_repeat(interval)
		return
	elseif M.main_timer:is_active() then
		return
	end

	M.main_timer:start(interval, interval, function()
		callback()
	end)
end

M.check_for_updates = vim.schedule_wrap(function()
	if not M.results or not M.picker then
		return
	end

	log.info("Checking for updates...")
	if not M.results.bufnr or not api.nvim_buf_is_loaded(M.results.bufnr) then
		M.main_timer:stop()
		-- M.main_timer:close()
	end

	local status = M.picker:tick(10)
	if status.changed or status.running or M.picker:should_rerender() then
		M.picker:force_rerender()
		M.tx.send()
	end

	if not (status.running or status.changed or M.picker:should_rerender()) and M.picker:total_items() > 0 then
		if not M.main_timer:is_closing() then
			M.main_timer:stop()
			-- M.main_timer:close()
		end
	end
end)

M.render_match_counts = vim.schedule_wrap(function()
	if not M.picker or not M.prompt then
		return
	end

	if not M.prompt.bufnr or not api.nvim_buf_is_loaded(M.prompt.bufnr) then
		return
	end

	M.picker:tick(10)
	local item_count = M.picker:total_items()
	local match_count = M.picker:total_matches()

	M.prompt:render_match_count(match_count, item_count)
end)

---@class PickerOptions
---@field cwd? string
---@field sort_direction? "ascending"|"descending"

---@param opts? PickerOptions
M.find = function(opts)
	M.original_winid = api.nvim_get_current_win()
	M.original_cursor = api.nvim_win_get_cursor(M.original_winid)

	M.results = Results()
	M.previewer = Previewer()
	M.initialize(opts)

	M.highlighter = Highlighter({
		picker = M.picker,
		results = M.results,
	})

	M.prompt = Prompt({
		input_options = {
			on_close = function()
				if M.main_timer and not M.main_timer:is_closing() then
					M.main_timer:stop()
					M.main_timer:close()
				end
				if M.picker then
					M.picker:restart()
				end
				if M.original_winid then
					api.nvim_set_current_win(M.original_winid)
				end
			end,
			on_submit = function()
				if M.main_timer and not M.main_timer:is_closing() then
					M.main_timer:stop()
					M.main_timer:close()
				end

				if M.picker:total_matches() == 0 then
					vim.notify("There's nothing to select", vim.log.levels.WARN)
					if M.original_winid then
						api.nvim_set_current_win(M.original_winid)
					end
				else
					local selection = M.picker:get_selection().path
					log.info("Input Submitted: " .. selection)

					if M.original_winid then
						api.nvim_set_current_win(M.original_winid)
					end
					vim.cmd.drop(string.format("%s", vim.fn.fnameescape(selection)))

					-- TODO: Figure out what to actually do here
					M.picker:restart()
				end
			end,
			on_change = M.process_input,
		},
	})

	M.prompt:map("n", "<Esc>", function()
		M.prompt:unmount()
	end, { noremap = true })

	M.prompt:map("i", "<Esc>", function()
		M.prompt:unmount()
	end, { noremap = true })

	M.prompt:map("i", { "<C-n>", "<Down>" }, function()
		M.picker:move_cursor_down()
		-- M.highlighter:highlight_selection()
		-- if M.picker:should_rerender() then
		M.tx.send()
		-- end
	end, { noremap = true })

	M.prompt:map("i", { "<C-p>", "<Up>" }, function()
		M.picker:move_cursor_up()
		-- M.highlighter:highlight_selection()
		-- if M.picker:should_rerender() then
		M.tx.send()
		-- end
	end, { noremap = true })

	local layout = Layout(
		{
			relative = "editor",
			position = "50%",
			size = {
				width = "80%",
				height = "80%",
			},
		},
		Layout.Box({
			Layout.Box(M.prompt, { size = { width = "100%", height = "3" } }),
			Layout.Box({
				Layout.Box(M.results, { size = "40%" }),
				Layout.Box(M.previewer, { size = "60%" }),
			}, { dir = "row", size = "100%" }),
		}, { dir = "col" })
	)

	M.results:on(event.BufWinEnter, function()
		local height = math.max(api.nvim_win_get_height(M.results.winid), 10)

		M.picker:update_window(height)
	end)

	M.prompt:on("VimResized", function()
		local height = math.max(api.nvim_win_get_height(M.results.winid), 10)

		M.picker:update_window(height)
	end)

	M.prompt:on(event.BufLeave, function()
		layout:unmount()
	end)

	layout:mount()

	local tx, rx = channel.counter()
	M.tx = tx

	local main_loop = a.void(function()
		log.info("Starting main loop...")

		M.set_interval(10, M.check_for_updates)

		await_schedule()

		while true do
			log.info("Looping...")
			rx.last()
			await_schedule()

			if not M.results.bufnr or not api.nvim_buf_is_loaded(M.results.bufnr) or not M.results.winid then
				return
			end

			if api.nvim_buf_line_count(M.results.bufnr) > 0 then
				M.highlighter:highlight_selection()
			end

			local status = M.picker:tick(10)
			if M.picker:should_rerender() or status.changed then
				log.info("trying to render in the main loop")
				log.info("Rendering with total matches: ", M.picker:total_matches())
				M.picker:drain_channel()
				M.render_match_counts()
				M.render_matches()
			end

			if M.picker:total_matches() > 0 then
				M.highlighter:highlight_selection()
				M.previewer:render(M.picker:get_selection().path)
			else
				M.previewer:clear()
			end
		end
	end)

	M.tx.send()
	main_loop()
end

function M.setup()
	api.nvim_create_user_command("Nucleo", function()
		M.find()
	end, {})
end

return M
