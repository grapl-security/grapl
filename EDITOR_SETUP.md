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

# Visual Studio Code

[Link][vsc]

## Rust Configuration

For Rust, the [Rust Analyzer Plugin][ra_vsc] is recommended.

Of particular note are the `checkOnSave` and `rustfmt` override
commands. As with many Rust projects, we still use [rustfmt][rustfmt],
[clippy][clippy], and `cargo check` for formatting and linting, but
the specific configurations are encapsulated into custom scripts that
we also use in CI. By also using them in an editor, we can ensure a
consistent experience for all uses of these tools.

Add the following to your workspace settings:

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

## Protobuf Configuration

We use [buf][buf] to lint our Protobuf definitions. The tool can be
downloaded from [the Github Releases page][buf_release], or by using
the [etc/install_buf.sh](./etc/install_buf.sh) script in this
repository.

The tool itself is configured via [buf.yaml](./buf.yaml).

As long as the binary is present in your `$PATH`, the [buf Visual Studio
Code plugin][buf_vsc] will automatically lint your Protobuf files
according to our configuration.

[vsc]: https://code.visualstudio.com/
[ra_vsc]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer
[clippy]: https://github.com/rust-lang/rust-clippy
[rustfmt]: https://github.com/rust-lang/rustfmt
[buf]: https://buf.build
[buf_release]: https://github.com/bufbuild/buf/releases
[buf_vsc]: https://marketplace.visualstudio.com/items?itemName=bufbuild.vscode-buf
