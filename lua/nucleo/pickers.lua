local Input = require("nui.input")
local Layout = require("nui.layout")
local Popup = require("nui.popup")
local event = require("nui.utils.autocmd").event
local Prompt = require("nucleo.prompt")
local Results = require("nucleo.results")
local a = require("plenary.async")
local await_schedule = a.util.scheduler
local channel = require("plenary.async.control").channel
local log = require("nucleo.log")
local nu = require("nucleo")
local debounce = require("throttle-debounce").debounce_trailing
local Entry = require("nucleo.entry")
local Highlighter = require("nucleo.highlighter")

local M = {}

M.results_bufnr = nil
M.picker = nil
M.results = nil
M.highlighter = nil
M.original_cursor = nil
M.tx = nil

M.render_matches = function()
	M.picker:tick(10)

	if M.picker:total_matches() == 0 then
		if vim.api.nvim_buf_is_loaded(M.results_bufnr) then
			vim.api.nvim_buf_set_lines(M.results_bufnr, 0, -1, false, {})
		end
	else
		local results = M.picker:current_matches()
		vim.iter(ipairs(results)):each(function(i, entry)
			return Entry(i, entry, M.results_bufnr):render()
		end)

		if not vim.tbl_isempty(results) then
			M.highlighter:highlight_selection()
		end
	end
end

---@param val string
M.process_input = debounce(function(val)
	M.picker:update_query(val)
	log.info("Updated input: " .. val)

	if M.tx then
		M.tx.send()
	end
end, 50)

M.initialize = function(opts)
	if not M.picker then
		M.picker = nu.Picker(opts)
	else
		M.picker:populate_files()
		M.picker:tick(10)

		vim.schedule(function()
			M.render_matches()
		end)
	end
end

---@class PickerConfig
---@field cwd? string

---@param opts? PickerConfig
M.find = function(opts)
	M.original_winid = vim.api.nvim_get_current_win()
	M.original_cursor = vim.api.nvim_win_get_cursor(M.original_winid)

	M.results = Results()
	M.initialize(opts)

	M.results_bufnr = M.results.bufnr
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
			winhighlight = "Normal:Normal,FloatBorder:Normal",
		},
	}, {
		prompt = "> ",
		default_value = "",
		on_close = function()
			if M.picker then
				M.picker:restart()
			end
			if M.original_winid then
				vim.api.nvim_set_current_win(M.original_winid)
			end
		end,
		---@param value string
		on_submit = function(value)
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
		M.highlighter:highlight_selection()
	end, { noremap = true })

	input:map("i", { "<C-p>", "<Up>" }, function()
		M.picker:move_cursor_up()
		M.highlighter:highlight_selection()
	end, { noremap = true })

	input:map("i", "<Esc>", function()
		input:unmount()
	end, { noremap = true })

	local layout = Layout(
		{
			relative = "editor",
			position = "50%",
			size = {
				width = "50%",
				height = "80%",
			},
		},
		Layout.Box({
			Layout.Box(input, { size = {
				width = "100%",
				height = "3",
			} }),
			Layout.Box(M.results, { size = "100%" }),
		}, { dir = "col" })
	)

	M.results:on(event.BufWinEnter, function()
		local height = math.max(vim.api.nvim_win_get_height(M.results.winid), 10)

		M.picker:update_window(height)
	end)

	input:on("VimResized", function(e)
		local height = math.max(vim.api.nvim_win_get_height(M.results.winid), 10)

		M.picker:update_window(height)
	end)

	input:on(event.BufLeave, function()
		layout:unmount()
	end)

	layout:mount()

	local tx, rx = channel.mpsc()
	M.tx = tx

	M.picker:update_query("")

	local main_loop = a.void(function()
		log.info("Starting main loop")
		await_schedule()

		while true do
			log.info("Looping...")
			rx.last()
			await_schedule()

			if not M.results_bufnr or not vim.api.nvim_buf_is_loaded(M.results_bufnr) then
				return
			end

			local _status = M.picker:tick(10)
			M.render_matches()
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
