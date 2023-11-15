if vim.g.nucleo_loaded then
	return
end

vim.g.nucleo_loaded = 1

require("nucleo").setup()
