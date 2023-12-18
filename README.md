```lua
  {
    "alexwu/nucleo.nvim",
    dependencies = { "nvim-lua/plenary.nvim", "MunifTanjim/nui.nvim" },
    config = true,
    opts = {},
    build = "just",
  }
```

```vim
Plug 'nvim-lua/plenary.nvim'
Plug 'MunifTanjim/nui.nvim'
Plug 'alexwu/nucleo.nvim', { 'do': 'just' }
```
