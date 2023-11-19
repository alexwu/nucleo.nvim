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

M.picker = nil
M.results = nil
M.highlighter = nil
M.original_cursor = nil
M.tx = nil
M.force_rerender = false

M.render_matches = function()
	M.results:render_entries(M.picker)
end

---@param val string
M.process_input = debounce(function(val)
	M.picker:update_query(val)
	M.force_rerender = true
	log.info("Updated input: " .. val)

	if M.tx then
		M.tx.send()
	end
end, 50)

M.initialize = function(opts)
	if not M.picker then
		M.picker = nu.Picker(opts)
		M.force_rerender = true
	else
		M.picker:populate_files()
		M.picker:tick(10)
		M.force_rerender = true

		M.tx.send()
	end
end

function M.set_interval(interval, callback)
	local timer = vim.uv.new_timer()
	timer:start(interval, interval, function()
		callback()
	end)

	return timer
end

M.check_for_updates = vim.schedule_wrap(function()
	if not M.results or not M.picker then
		return
	end

	log.info("Checking for updates...")
	if not M.results.bufnr or not api.nvim_buf_is_loaded(M.results.bufnr) then
		M.main_timer:stop()
		M.main_timer:close()
	end

	local status = M.picker:tick(10)
	if status.changed or status.running or M.picker:should_update() then
		M.force_rerender = true
		M.tx.send()
	end

	if not (status.running or status.changed or M.picker:should_update()) then
		M.main_timer:stop()
		M.main_timer:close()
	end
end)

M.render_match_counts = vim.schedule_wrap(function()
	if not M.picker or not M.prompt then
		return
	end

	if not M.prompt.bufnr or not api.nvim_buf_is_loaded(M.prompt.bufnr) then
		if M.counter_timer and not M.counter_timer:is_closing() then
			M.counter_timer:stop()
			M.counter_timer:close()
		end
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
				if M.picker then
					M.picker:restart()
				end
				if M.original_winid then
					api.nvim_set_current_win(M.original_winid)
				end
			end,
			on_submit = function()
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
		M.force_rerender = true
		M.tx.send()
	end, { noremap = true })

	M.prompt:map("i", { "<C-p>", "<Up>" }, function()
		M.picker:move_cursor_up()
		M.force_rerender = true
		M.tx.send()
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

		M.main_timer = M.set_interval(100, M.check_for_updates)
		-- M.counter_timer = M.set_interval(1000, M.render_match_counts)

		await_schedule()

		while true do
			log.info("Looping...")
			rx.last()
			await_schedule()

			if not M.results.bufnr or not api.nvim_buf_is_loaded(M.results.bufnr) or not M.results.winid then
				return
			end

			local status = M.picker:tick(10)
			if M.force_rerender or status.changed then
				log.info("trying to render in the main loop")
				M.render_match_counts()
				M.render_matches()
				M.force_rerender = false
			end

			M.highlighter:highlight_selection()
			if M.picker:total_matches() > 0 then
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
