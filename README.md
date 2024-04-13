# lspml

A work-in-progress language server for the sitepark markup language (spml).

## features

- go to definition for variables and `<sp:include>` tag `uri` attributes
- hover for documentation of most tags and attributes
- diagnostics on:
    - syntax errors
    - spel syntax errors in attributes values
    - misplaced, unclosed and deprecated tags
    - duplicate, required and deprecated attributes / tag-bodies
    - nonexistent files in `<sp:include>` and similar tags
- completion for:
    - tags
    - `</`, closing the last unclosed tag
    - attributes
    - attribute values that either:
        - have a fixed set of possible values
        - point to another spml file
- semantic highlighting for attribute values that expect:
    - conditions
    - expressions
    - identifiers
    - objects
    - regular expressions
    - text
    - uris
    - to be comparable (for `sp:if` and `sp:elseif` `eq`/`gt`/...)
- code actions to:
    - generate a default file header
    - split `sp:if` `condition` into `name` and `eq`/`gt`/`isNull`/...
    - join `sp:if` `name` and `eq`/`gt`/`isNull`/... into `condition`

## commandline

```
Usage: lspml [OPTIONS]

Options:
      --log-file <LOG_FILE>
      --log-level <LOG_LEVEL>        [default: INFO]
      --modules-file <MODULES_FILE>
  -h, --help                         Print help
```

The `modules-file` is a `json` file, in which module names can be mapped to local repositories like so:
```json
{
    "test-module": {
        "path": "/path/to/this/repo/lspml/test-spml/test-module/src/main/webapp"
    },
    "sitekit-module": {
        "path": "/path/to/this/repo/lspml/test-spml/sitekit-module/src/main/webapp"
    }
}
```

## build

Currently you will have to be able to access [tree-sitter-spml](https://github.com/DrWursterich/tree-sitter-spml) via ssh in order to be able to build this with:


```bash
cargo +nightly build
```

## install

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

