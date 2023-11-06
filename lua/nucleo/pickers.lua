local Input = require("nui.input")
local Layout = require("nui.layout")
local Popup = require("nui.popup")
local event = require("nui.utils.autocmd").event
local Prompt = require("nucleo.prompt")
local Results = require("nucleo.results")
local a = require("plenary.async")
local log = require("nucleo.log")
local nu = require("nucleo")
local debounce = require("throttle-debounce").debounce_trailing

local M = {}

M.results_bufnr = nil
M.selection_index = 1
M.co = nil
M.picker = nil

M.process_input = debounce(function(val)
	M.picker:update_query(val)

	local status = M.picker:tick(10)
	if status.changed then
		local results = M.picker:current_matches()
		vim.schedule(function()
			if M.results_bufnr and vim.api.nvim_buf_is_loaded(M.results_bufnr) then
				vim.api.nvim_buf_set_lines(M.results_bufnr, 0, -1, false, results)
			end
		end)
	end
end, 50)

M.initialize = function()
	if not M.picker then
		M.picker = nu.Picker()
	else
		vim.schedule(function()
			M.picker:populate_files()
		end)
	end
end

function M.setup()
	vim.api.nvim_create_user_command("Nucleo", function()
		local results = Results()

		M.initialize()

		M.results_bufnr = results.bufnr

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
			end,
			on_submit = function(value)
				local selection = M.picker:get_selection()
				log.info("Input Submitted: " .. selection)
				vim.cmd.edit(string.format("%s", vim.fn.fnameescape(selection)))
			end,
			on_change = M.process_input,
		})

		input:map("n", "<Esc>", function()
			input:unmount()
		end, { noremap = true })

		input:map("i", "<C-n>", function()
			M.picker:move_cursor_down()
		end, { noremap = true })
		input:map("i", "<C-p>", function()
			M.picker:move_cursor_up()
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
				Layout.Box(results, { size = "100%" }),
				Layout.Box(input, { size = {
					width = "100%",
					height = "3",
				} }),
			}, { dir = "col" })
		)

		input:on(event.BufWinEnter, function()
			-- vim.print(vim.bo.filetype)
			-- 	log.info("Before init")
			-- 	vim.schedule(function()
			-- M.picker:populate_picker(vim.loop.cwd())
			-- 		log.info("After init")
		end)
		-- end)

		input:on(event.BufLeave, function()
			input:unmount()
		end)

		layout:mount()
	end, {})
end

M.setup()

return M
