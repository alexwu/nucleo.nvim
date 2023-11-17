local Entry = require("nucleo.entry")
local NuiPopup = require("nui.popup")

---@class Results: NuiPopup
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
		position = { row = 0, col = "100%" },
		size = { width = 10, height = 1 },
		win_options = {
			winhighlight = "Normal:Normal,FloatBorder:FloatBorder",
		},
		options = {},
	})

	self.sort_direction = opts.sort_direction or "descending"

	Results.super.init(self, popup_options)
end

function Results:render_entries(picker)
	picker:tick(10)

	if picker:total_matches() == 0 then
		if vim.api.nvim_buf_is_loaded(self.bufnr) then
			vim.api.nvim_buf_set_lines(self.bufnr, 0, -1, false, {})
		end
	else
		local results = picker:current_matches()
		vim.iter(ipairs(results)):each(function(i, entry)
			return Entry(i, entry, self.bufnr):render()
		end)
	end
end

return Results
