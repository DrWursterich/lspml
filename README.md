# lspml

A work-in-progress language server for the sitepark markup language (spml).

## features

- go to definition for variables and `<sp:include>` tag `uri` attributes
- hover for documentation of
    - most tags
    - most attributes
    - attribute enum values
    - global functions in spel attribute values
- diagnostics on:
    - syntax errors
    - misplaced, unclosed and deprecated tags
    - duplicate, required and deprecated attributes / tag-bodies
    - nonexistent files in `<sp:include>` and similar tags
    - sitepark expression language (spel):
        - syntax errors
        - nonexistent global functions
        - incorrect argument counts for global functions
- completion for:
    - tags
    - `</`, closing the last unclosed tag
    - attributes
    - attribute values that either:
        - have a fixed set of possible values
        - point to another spml file
        - refer to an spml module
- semantic highlighting for attribute values that expect:
    - conditions
    - expressions
    - identifiers
    - objects
    - regular expressions
    - text
    - uris
    - to be comparable (for `<sp:if>` and `<sp:elseif>` `eq`/`gt`/...)
- code actions to:
    - generate a default file header
    - fix small spel syntax errors (`quickfix`)
    - fix all `quickfix`-able errors at once (`source.fixAll`)
    - split `<sp:if>` `condition` into `name` and `eq`/`gt`/`isNull`/...
    - join `<sp:if>` `name` and `eq`/`gt`/`isNull`/... into `condition`

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
        "path": "/absolute/path/to/some/test-module/src/main/webapp"
    },
    "sitekit-module": {
        "path": "/absolute/path/to/some/other/sitekit-module/src/main/webapp"
    }
}
```

## build

```bash
cargo +nightly build --release
```

## install

### nvim

As of now there is no `lsp-config` configuration for lspml, so attatching it has to be done manually:
```lua
-- manually attach lspml ls
vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter" }, {
    pattern = { "*.spml" },
    callback = function(ev)
        vim.lsp.start({
            name = 'lspml',
            cmd = {
                '/path/to/lspml',
                '--modules-file', '/path/to/module_mappings.json', --optional
                '--log-file', '/path/to/lspml.log.json',           --optional
                '--log-level', 'INFO',                             --optional
            },
            root_dir = vim.fs.dirname(vim.fs.find({ 'src' }, { upward = true })[1]),
        })
        vim.api.nvim_create_autocmd('LspAttach', {
            callback = function(client, bufnr)
                -- register custom keymaps, omnifunc, formatting, ...
            end,
        })
    end
})
```

The `lspml` executable is best found (after building) with

```bash
find . -name lspml -executable
```

