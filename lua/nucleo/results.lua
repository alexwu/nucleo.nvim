local Entry = require("nucleo.entry")
local NuiPopup = require("nui.popup")
local log = require("nucleo.log")

---@class Results: NuiPopup
---@field sort_direction "ascending"|"descending"
local Results = NuiPopup:extend("Results")

---@class ResultsOptions
---@field sort_direction? "ascending"|"descending"
---@field popup_options? nui_popup_options

---@param opts? ResultsOptions
function Results:init(opts)
	opts = vim.F.if_nil(opts, { popup_options = {} })
	local popup_options = vim.tbl_deep_extend("force", opts.popup_options or {}, {
		border = "rounded",
		focusable = true,
		buf_options = {
			filetype = "NucleoResults",
		},
		win_options = {
			winhighlight = "Normal:Normal,FloatBorder:FloatBorder",
		},
		options = {},
	})

	-- self.sort_direction = opts.sort_direction or "descending"
	self.sort_direction = "ascending"

	Results.super.init(self, popup_options)
end

---@param bufnr number
---@param height number
local function clear_buffer(bufnr, height)
	local empty_lines = {}
	for _ = 1, height do
		table.insert(empty_lines, #empty_lines + 1, "")
	end

	vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, empty_lines)
end

function Results:clear_buffer()
	local height = math.max(vim.api.nvim_win_get_height(self.winid), 10)
	clear_buffer(self.bufnr, height)
end

function Results:render_entries(picker)
	if not self.winid then
		return
	end

	-- picker:tick(10)

	if picker:total_matches() == 0 then
		if vim.api.nvim_buf_is_loaded(self.bufnr) and vim.api.nvim_win_is_valid(self.winid) then
			Results.clear_buffer(self)
		end
	else
		local height = vim.api.nvim_win_get_height(self.winid)
		Results.clear_buffer(self)
		local results = picker:current_matches()
		vim.iter(ipairs(results)):each(function(i, entry)
			local index = i
			if self.sort_direction == "ascending" then
				index = height - i + 1
				log.info("trying to render index", index)
			end
			return Entry(index, entry, self.bufnr):render()
		end)
	end
end

return Results
