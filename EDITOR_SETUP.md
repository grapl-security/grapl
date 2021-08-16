Setting up your Editor or IDE to work on Grapl
==============================================

Here, we document tips for setting up various editors and IDEs for
efficiently working on the Grapl codebase. The presence of a
particular tool here is not an endorsement. Similarly, the absence of
a tool doesn't mean it can't be used. This is just an informative
document to help new contributors quickly get up to speed.

These configurations are not necessarily complete (this document is a
work in progress), and they are not necessarily the _only_ way these
tools can be configured. They are, however, configurations that work.

If you have configuration instructions for an editor or IDE not
covered here, please feel free to submit a
[PR](https://github.com/grapl-security/grapl/pulls)!

# Rust Configuration

## Visual Studio Code

For Rust, the [Rust Analyzer Plugin][ra_vsc] is recommended.

Add the following to your workspace settings (note that you will have
to change all `/path/to/your/grapl/repo` paths as appropriate for your
workstation):

``` json
"settings": {
    "editor.formatOnSave": true,
    "editor.formatOnPaste": true,
    "[rust]": {
        "editor.defaultFormatter": "matklad.rust-analyzer",
    },
    "rust-analyzer.linkedProjects": [
        "src/rust/Cargo.toml"
    ],
    "rust-analyzer.checkOnSave.enable": true,
    "rust-analyzer.checkOnSave.overrideCommand": [
        "/path/to/your/grapl/repo/src/rust/bin/lint",
        "json"
    ],
    "rust-analyzer.rustfmt.overrideCommand": [
        "/path/to/your/grapl/repo/src/rust/bin/format",
        "--editor"
    ]
}
```

Of particular note are the `checkOnSave` and `rustfmt` override
commands. As with many Rust projects, we still use [rustfmt][rustfmt],
[clippy][clippy], and `cargo check` for formatting and linting, but
the specific configurations are encapsulated into custom scripts that
we also use in CI. By also using them in an editor, we can ensure a
consistent experience for all uses of these tools.

# Python Configuration

Our Python monorepo relies heavily on the
[Pants](https://pantsbuild.org) build system. The `pants` script is
included in this repository (as is standard practice for Pants), so no
additional tooling needs to be downloaded and installed for it.

Pants knows all about how our Python code is structured, how it
interrelates, and what code depends on it. However, we have a bit of
work to do before that knowledge can be usefully exposed to our
editors.

## Common Setup for Pyright

We have had good experience with [Pyright][pyright] for integrating
our editors with our Python code. Plugins are available for a variety
of editors, but some common setup must be done for each.

First, you must set up an appropriate virtual environment for our
3rd-party Python dependencies. Run the following command.

```sh
make populate-venv
```
Follow the instructions in the output of that command to activate the
virtual environment.

As our dependencies are updated, you can re-run this command to ensure
your virtual environment is up-to-date.

Next, you'll need to create a `pyrightconfig.json` file. Because of
how our Python code is currently laid out, this can be a bit
complex. Fortunately, this can be driven by the information from our
Pants configuration file ([pants.toml](./pants.toml).

To generate this file, run the following command (this requires that
the virtual environment you created above is active):

```sh
build-support/editor_setup.py pyright generate
```
This will create a functional configuration for you. You may then modify it
as you like.

As we add to and modify our Python code (including its organization on
disk), this `pyrightconfig.json` fill will need to be updated to
reflect these changes. Automation saves the day once again,
however. Running the following command will update only the parts of
the configuration file that are impacted by our code layout:

```sh
build-support/editor_setup.py pyright update
```
Any customizations outside of that small amount of information will be
preserved.

(For further details, feel free to execute any of those
`editor_setup.py` commands with the `--help` option.

Once you have a configuration file in place, you can proceed on to
editor-specific configuration in the sections below.

### Visual Studio Code

While you can use either the [Pyright][pyright-vsc] or
[Pylance][pylance-vsc] plugins, Pylance is recommended, as it is based
in part on Pyright, and is overall more advanced. These plugins will
automatically configure themselves according to the
`pyrightconfig.json` file we generated above.

### Emacs

Use [lsp-mode][lsp-mode-emacs] along with
[lsp-pyright][lsp-pyright-emacs]. The necessary configuration will be
picked up automatically from the `pyrightconfig.json` file we
generated above.

[clippy]: https://github.com/rust-lang/rust-clippy
[lsp-mode-emacs]: https://github.com/emacs-lsp/lsp-mode
[lsp-pyright-emacs]: https://github.com/emacs-lsp/lsp-pyright
[pylance-vsc]: https://marketplace.visualstudio.com/items?itemName=ms-python.vscode-pylance
[pyright]: https://github.com/Microsoft/pyright
[pyright-vsc]: https://marketplace.visualstudio.com/items?itemName=ms-pyright.pyright
[ra_vsc]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer
[rustfmt]: https://github.com/rust-lang/rustfmt
[vsc]: https://code.visualstudio.com/

## TypeScript Setup

In an effort to maintain consistency among the files in our front-end codebase, we use the VS Code extension, "Prettier". 

The reason why Grapl uses a consistent formatter on our front-end is because certain elements require no whitespace between tags. 

Sometimes auto-formatters add extra spaces, which may occasionally result in bugs. 

If you don't have the VSCode extension, "Prettier", please install it by clicking the "Extensions" icon on the left panel in VSCode.

Then search for "Prettier", and clicking the "Install" button.

To update your Prettier extension settings in VSCode, navigate to:
- File > Settings > Extensions > Prettier
- On the upper right hand corner of the VSCode IDE underneath the close(X) button, click the “Open Settings (JSON) button
- Ensure your settings match the file below: 

```
{
    "window.zoomLevel": 0,
    "javascript.updateImportsOnFileMove.enabled": "always",
    "typescript.updateImportsOnFileMove.enabled": "always",
    "tabnine.experimentalAutoImports": true,
    "diffEditor.ignoreTrimWhitespace": false,
    "[javascript]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    },
    "[typescriptreact]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    },
    "prettier.useTabs": true,
    "[typescript]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    },
    "[json]": {
        "editor.defaultFormatter": "esbenp.prettier-vscode"
    }
}
```
