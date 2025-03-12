# lspml

A work-in-progress language server for the sitepark markup language (spml).

## features

- go to definition for variables and `<sp:include>` tag `uri` attributes
- hover for documentation of
    - most tags
    - most attributes
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
    - the last unclosed tag
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

### static code analisis

this project also contains the `analyze` command to invoke its diagnostic statically.

```
#$ lspml analyze --directory src/main/webapp/ --module-file module_mappings.json
src/main/webapp/templates/sectionTypes/bad.spml
[CRITICAL] missing required attribute "name" (MISSING_VALUE) on line 10
[ERROR]    included file "/functions/missing.spml" does not exist (MISSING_FILE) on line 24
[WARNING]  attribute "object" is useless without attribute "action" containing one of these values: [put, putNotEmpty, putAll, merge] (SUPERFLUOUS_VALUE) on line 12

src/main/webapp/templates/sectionTypes/veryBad.spml
[CRITICAL] missing atleast one header. Try generating one with the "refactor.generate_default_headers" code-action (MISSING_HEADER) on line 1
[CRITICAL] syntax error: unexpected "/" (SYNTAX_ERROR) on line 2
[CRITICAL] syntax error: unexpected ":" (SYNTAX_ERROR) on line 2
```

When using the `gitlab` format for [code-quality](https://docs.gitlab.com/ci/testing/code_quality) `lspml analyze` has to be executed from the projects root directory such that the relative paths in the json output align. It may also be usefull to add `--ignore UNKNOWN_MODULE` to skip validation of dependencies.

__disclaimer__: this functionallity will probably become a separate project/binary when it leaves the experimental state!

## commandline

```
Usage: lspml [OPTIONS] [COMMAND]

Commands:
  analyze
  help     Print this message or the help of the given subcommand(s)

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

## install

You can install the binaries without checking this project out via:

```bash
cargo +nightly install --git https://github.com/DrWursterich/lspml.git
```

Or if you have tinkered with it:

```bash
cargo +nightly install --path .
```

## build

```bash
cargo +nightly build --release
```

This creates the executables `lspml` and `lspml-analyze` at `./target/release/` or `./target/debug/` if not using `--release`

## use

### neovim

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

### sublime text

Install [the lsp package](https://lsp.sublimetext.io/) and follow their setup instructions. A configuration for `lspml` should look similar to this:
```json
{
    "clients": {
        "lspml": {
            "enabled": true,
            "command": [
                "/path/to/lspml",
                "--modules-file", "/path/to/module_mappings.json",
                "--log-file", "/path/to/lspml/lspml.log.json",
                "--log-level", "INFO"
            ],
            "selector": "text.html.jsp"
        }
    }
}
```

### visual studio code

The [spml-vscode](https://github.com/sitepark/spml-vscode) extension wraps `lspml` and provides all of it's features.

