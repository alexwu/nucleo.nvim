local Input = require("nui.input")
local Layout = require("nui.layout")
local event = require("nui.utils.autocmd").event
local Results = require("nucleo.results")
local a = require("plenary.async")
local await_schedule = a.util.scheduler
local channel = require("plenary.async.control").channel
local log = require("nucleo.log")
local nu = require("nucleo")
local debounce = require("throttle-debounce").debounce_trailing
local Highlighter = require("nucleo.highlighter")
local Previewer = require("nucleo.previewer")
local Text = require("nui.text")

local M = {}

M.picker = nil
M.results = nil
M.highlighter = nil
M.original_cursor = nil
M.tx = nil
M.should_rerender = false

M.render_matches = function()
	M.results:render_entries(M.picker)
end

---@param val string
M.process_input = debounce(function(val)
	M.picker:update_query(val)
	M.should_rerender = true
	log.info("Updated input: " .. val)

	if M.tx then
		M.tx.send()
	end
end, 50)

M.initialize = function(opts)
	if not M.picker then
		M.picker = nu.Picker(opts)
		M.should_rerender = true
	else
		M.picker:populate_files()
		M.picker:tick(10)
		M.should_rerender = true

		M.tx.send()
	end
end

---@class PickerOptions
---@field cwd? string
---@field sort_direction? "ascending"|"descending"

---@param opts? PickerOptions
M.find = function(opts)
	M.original_winid = vim.api.nvim_get_current_win()
	M.original_cursor = vim.api.nvim_win_get_cursor(M.original_winid)

	M.results = Results()
	M.previewer = Previewer()
	M.initialize(opts)

	M.highlighter = Highlighter({
		picker = M.picker,
		bufnr = M.results.bufnr,
	})

	local input = Input({
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
	}, {
		prompt = Text(" ", "TelescopePromptPrefix"),
		default_value = "",
		on_close = function()
			if M.picker then
				M.picker:restart()
			end
			if M.original_winid then
				vim.api.nvim_set_current_win(M.original_winid)
			end
		end,
		on_submit = function()
			if M.picker:total_matches() == 0 then
				vim.notify("There's nothing to select", vim.log.levels.WARN)
				if M.original_winid then
					vim.api.nvim_set_current_win(M.original_winid)
				end
			else
				local selection = M.picker:get_selection().path
				log.info("Input Submitted: " .. selection)

				if M.original_winid then
					vim.api.nvim_set_current_win(M.original_winid)
				end
				vim.cmd.drop(string.format("%s", vim.fn.fnameescape(selection)))

				-- TODO: Figure out what to actually do here
				M.picker:restart()
			end
		end,
		on_change = M.process_input,
	})

	input:map("n", "<Esc>", function()
		input:unmount()
	end, { noremap = true })

	input:map("i", { "<C-n>", "<Down>" }, function()
		M.picker:move_cursor_down()
		-- M.highlighter:highlight_selection()
		M.tx.send()
	end, { noremap = true })

	input:map("i", { "<C-p>", "<Up>" }, function()
		M.picker:move_cursor_up()
		-- M.highlighter:highlight_selection()
		M.tx.send()
	end, { noremap = true })

	input:map("i", "<Esc>", function()
		input:unmount()
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
			Layout.Box(input, { size = { width = "100%", height = "3" } }),
			Layout.Box({
				Layout.Box(M.results, { size = "40%" }),
				Layout.Box(M.previewer, { size = "60%" }),
			}, { dir = "row", size = "100%" }),
		}, { dir = "col" })
	)

	M.results:on(event.BufWinEnter, function()
		local height = math.max(vim.api.nvim_win_get_height(M.results.winid), 10)

		M.picker:update_window(height)
	end)

	input:on("VimResized", function()
		local height = math.max(vim.api.nvim_win_get_height(M.results.winid), 10)

		M.picker:update_window(height)
	end)

	input:on(event.BufLeave, function()
		layout:unmount()
	end)

	layout:mount()

	local tx, rx = channel.counter()
	M.tx = tx

	function M.set_interval(interval, callback)
		M.timer = vim.uv.new_timer()
		M.timer:start(interval, interval, function()
			callback()
		end)
		return M.timer
	end

	M.check_for_updates = vim.schedule_wrap(function()
		log.info("trying wait callback")
		if not M.results.bufnr or not vim.api.nvim_buf_is_loaded(M.results.bufnr) then
			M.timer:stop()
			M.timer:close()
		end

		local status = M.picker:tick(10)
		log.info("running: ", status.running)
		log.info("changed: ", status.changed)
		log.info("should_update: ", M.picker:should_update())
		if status.changed or status.running or M.picker:should_update() then
			M.should_rerender = true
			M.tx.send()
		end

		if not (status.running or status.changed or M.picker:should_update()) then
			M.timer:stop()
			M.timer:close()
		end
	end)

	local main_loop = a.void(function()
		log.info("Starting main loop")

		M.set_interval(100, M.check_for_updates)

		log.info("Right after the wait call")

		await_schedule()

		while true do
			log.info("Looping...")
			rx.last()
			await_schedule()

			if not M.results.bufnr or not vim.api.nvim_buf_is_loaded(M.results.bufnr) then
				return
			end

			local status = M.picker:tick(10)
			if M.should_rerender or status.changed then
				log.info("trying to render in the main loop")
				M.render_matches()
				M.should_rerender = false
			end

			M.highlighter:highlight_selection()
			if M.picker:total_matches() > 0 then
				M.previewer:render(M.picker:get_selection().path)
			end
		end
	end)

	M.tx.send()
	main_loop()
end

function M.setup()
	vim.api.nvim_create_user_command("Nucleo", function()
		M.find()
	end, {})
end

return M
