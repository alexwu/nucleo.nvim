local M = {}

M.find_files = require("nucleo.sources.files").find_files
M.git_status = require("nucleo.sources.git").git_status
M.diagnostics = require("nucleo.sources.diagnostics").diagnostics

return M
