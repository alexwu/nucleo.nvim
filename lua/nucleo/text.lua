local NuiText = require("nui.text")

---@class Text: NuiText
---@field super NuiText
local Text = NuiText:extend("Text")

---@class TextOptions
---@field separator? string

---@param content string|NuiText text content or NuiText object
---@param extmark? string|nui_text_extmark highlight group name or extmark options
function Text:init(content, extmark)
	Text.super.init(self, content, extmark)
end

return Text
