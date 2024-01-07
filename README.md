# lspml

A proof-of-concept language server for the sitepark markup language (spml).

## features

- go to definition for variables and `<sp:include/>` tags

## building

```bash
cargo build
```

## installation

### nvim

As of now there is no `lsp-config` configuration for lspml, so attatching it has to be done manually:
```lua
	-- manually attach lspml ls
	vim.api.nvim_create_autocmd({"BufEnter", "BufWinEnter"}, {
		pattern = {"*.spml"},
		callback = function(ev)
			-- for debugging purposes
			-- vim.lsp.set_log_level("debug")
			vim.lsp.start({
				name = 'lspml',
				cmd = {'lspml'},
				root_dir = vim.fs.dirname(vim.fs.find({'src'}, {upward = true})[1]),
			})
			vim.api.nvim_create_autocmd('LspAttach', {
				callback = function(args)
					print(vim.inspect(args))
					vim.bo[args.buf].omnifunc = 'v:lua.vim.lsp.omnifunc'
					-- register custom keymaps
					vim.api.nvim_buf_set_keymap(
							0,
							'n',
							'gd',
							'',
							{
								callback = vim.lsp.buf.definition,
								noremap = true,
								silent = true,
								expr = false
							})
				end,
			})
		end
	})

```

The `lspml` executable is best found with

```bash
find . -name lspml -executable
```
