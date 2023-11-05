local Input = require("nui.input")
local Layout = require("nui.layout")
local Popup = require("nui.popup")
local event = require("nui.utils.autocmd").event
local Prompt = require("nucleo.prompt")
local Results = require("nucleo.results")
local a = require("plenary.async")
local log = require("nucleo.log")
local nu = require("nucleo")

local M = {}

M.results_bufnr = nil
M.selection_index = 1
M.co = nil

M.process_input = function(val)
	-- log.info("Before coroutine")
	-- if M.co then
	-- 	log.info(M.co)
	-- 	log.info(coroutine.status(M.co))
	-- end

	-- M.co = coroutine.create(function()
	-- a.void(function()
	-- log.info("Before fuzzy")

	-- local results = require("nucleo").fuzzy_file(val, vim.loop.cwd())

	M.picker:update_query(val)

	-- log.info("Got results")
	-- vim.print(M.results_bufnr)

	local status = M.picker:tick(10)
	log.info(status.changed)
	-- log.info(status.changed)
	if status.changed then
		-- local results = {}
		local results = M.picker:current_matches()
		vim.schedule(function()
			if M.results_bufnr and vim.api.nvim_buf_is_loaded(M.results_bufnr) then
				vim.api.nvim_buf_set_lines(M.results_bufnr, 0, -1, false, results)
			end
		end)
	end
end

M.initialize_files = function()
	if M.picker then
		M.co = coroutine.create(function()
			vim.schedule(function()
				M.picker:populate_picker()
			end)
		end)
		coroutine.resume(M.co)
	end
end

-- coroutine.resume(M.co)
-- end

M.picker = nil
-- M.picker = nu.Picker()

function M.setup()
	vim.api.nvim_create_user_command("Nucleo", function()
		local results = Results()

		M.picker = nu.Picker()
		M.results_bufnr = results.bufnr
		M.selection_index = 1

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
				-- require("nucleo").restart_picker()
			end,
			on_submit = function(value)
				print("Input Submitted: " .. value)
			end,
			on_change = M.process_input,
		})

		input:map("n", "<Esc>", function()
			input:unmount()
		end, { noremap = true })

		input:map("n", "<C-n>", function()
			M.selection_index = M.selection_index - 1
		end, { noremap = true })
		input:map("n", "<C-p>", function()
			M.selection_index = M.selection_index + 1
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
