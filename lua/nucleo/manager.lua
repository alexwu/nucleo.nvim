local Layout = require("nui.layout")
local log = require("nucleo.log")
local nu = require("nucleo")
local channel = require("nucleo.async").channel
local Highlighter = require("nucleo.highlighter")
local Previewer = require("nucleo.previewer")
local Prompt = require("nucleo.prompt")
local Results = require("nucleo.results")

local api = vim.api

---@class Manager: Object
local Manager = require("plenary.class"):extend()

function Manager:new(opts)
	-- TODO: These need to go somewhere else
end

return Manager
