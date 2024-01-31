local M = {}

M.find_files = require("nucleo.sources.files").find_files
M.git_status = require("nucleo.sources.git").git_status
M.git_hunks = require("nucleo.sources.git").git_hunks
M.diagnostics = require("nucleo.sources.diagnostics").diagnostics

return M
