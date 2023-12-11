```lua
  {
    "alexwu/nucleo.nvim",
    dependencies = { "nvim-lua/plenary.nvim", "MunifTanjim/nui.nvim", "nvim-tree/nvim-web-devicons" },
    config = true,
    opts = {},
    build = "just",
  }
```

```vim
Plug 'nvim-lua/plenary.nvim'
Plug 'MunifTanjim/nui.nvim'
Plug 'nvim-tree/nvim-web-devicons'
Plug 'alexwu/nucleo.nvim', { 'do': 'just' }
```
