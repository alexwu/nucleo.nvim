local Input = require("nui.input")
local Layout = require("nui.layout")
local Popup = require("nui.popup")
local event = require("nui.utils.autocmd").event

local M = {}

M.results_bufnr = nil
M.selection_index = 1

vim.api.nvim_create_user_command("Nucleo", function()
	local files = require("nucleo").files(vim.loop.cwd())

	local results = Popup({
		border = "rounded",
		size = {
			width = 80,
			height = 40,
		},
		enter = false,
		focusable = false,
		buf_options = {
			modifiable = true,
			readonly = false,
		},
	})

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
			print("Input Closed!")
		end,
		on_submit = function(value)
			print("Input Submitted: " .. value)
		end,
		on_change = function(value)
			local results = require("nucleo").fuzzy_match(value, files)
			if M.results_bufnr and vim.api.nvim_buf_is_loaded(M.results_bufnr) then
				vim.defer_fn(function()
					vim.api.nvim_buf_set_lines(M.results_bufnr, 0, -1, false, results)
				end, 0)
			end
		end,
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
			size = { height = "80%", width = "80%" },
		},
		Layout.Box({
			Layout.Box(results, { grow = 1 }),
			Layout.Box(input, { size = "25%" }),
		}, { dir = "col" })
	)

	-- for _, popup in pairs(popups) do
	--   popup:on("BufLeave", function()
	--     vim.schedule(function()
	--       local curr_bufnr = vim.api.nvim_get_current_buf()
	--       for _, p in pairs(popups) do
	--         if p.bufnr == curr_bufnr then
	--           return
	--         end
	--       end
	--       layout:unmount()
	--     end)
	--   end)
	-- end

	-- mount/open the component
	-- input:mount()
	-- layout:mount()

	-- vim.api.nvim_buf_attach(input.bufnr, false, {
	--   -- on_bytes = function(_, handle, changedtick, start_row, start_column, byte_offset, old_end_row, old_end_col other_last)
	--   --   local lines = vim.api.nvim_buf_get_lines(handle, start, last, true)
	--   --   vim.print(lines)
	--   -- end,
	--   on_lines = function(_, handle, changedtick, start, last, other_last)
	--     local lines = vim.api.nvim_buf_get_lines(handle, start, last, true)
	--     -- vim.print(lines)
	--     local prompt = vim.trim(string.gsub(lines[1], ">", "", 1))
	--     local results = require("nucleo").fuzzy_match(prompt, files)
	--     vim.print(results)
	--   end,
	--
	--   -- on_detach = function()
	--   --   self:_detach()
	--   -- end,
	-- })

	-- unmount component when cursor leaves buffer
	input:on(event.BufLeave, function()
		input:unmount()
	end)
end, {})

return M
