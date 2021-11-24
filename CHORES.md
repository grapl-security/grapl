Chores
======

Here are instructions for various chores we should do periodically, as well as
background for _why_ we do them.

These are manual tasks now, but would be ideal candidates for future automation.

# Rust Chores

These chores pertain to our Rust codebase.

## Update stable Rust for the project as a whole
The Rust team releases a new stable version of Rust every 6 weeks, with
announcements posted to the [Rust Blog][blog]. To ensure we are benefitting from
all the improvements to the language, we should plan to update the version of
Rust we are using periodically.

To do this, you must update the `channel` entry in [our toolchain
file](./src/rust/rust-toolchain.toml) to the appropriate version identifier.

Note that there may be new compiler warnings to address after updating the
version of Rust we use; please be sure to take care of these at this time.

## Update unstable Rust for formatting
We also currently make us of _unstable_ Rust (more specifically, unstable
`rustfmt`) for formatting our code. Rather than using the toolchain file (which
applies to _everything_), we specify this particular constraint in the
[formatting script](./src/rust/bin/format) via the [`RUSTUP_TOOLCHAIN`
environment variable][env]. To ensure stability across time and consistency
across team members, we specify a particular nightly release to use.

New versions of the unstable toolchain are released every night, but we don't
want to make corresponding changes in our codebase that frequently. Instead,
when we update our main Rust toolchain (as described above), we should also
update which unstable nightly Rust we're using for our formatting; a good option
would be to use the one released last night!

Note, however, that because unstable Rust is, well, _unstable_, there is no
guarantee that there is actually a usable build of `rustfmt` for any given
nightly Rust release (this is less common that it has been in the past, but it
can still happen). Therefore, when selecting a new nightly Rust, make sure the
`rustfmt` component is actually available in that release. To help, consult the
[Rustup Components History][history] page.

Since we currently only need an unstable Rust for specific unstable `rustfmt`
configuration options, you should take this opportunity to see which of [the
options we use][rustfmt_toml] have been stabilized since we last
checked. Once they have all been stabilized, we will no longer need to use
unstable Rust for anything. The current status of any `rustfmt` configuration
option can be discovered [here][options].

Also see [below](#update-rust-formatting-for-the-entire-codebase), as well!

## Update Rust formatting for the entire codebase
After [updating the version of nightly Rust that `rustfmt`
uses](#update-unstable-rust-for-formatting), or modifying any of the [`rustfmt`
configuration options we use][rustfmt_toml], we will need to update the code
pick up any changes.

To do this, run the following command and commit the results in the same PR as
the nightly / configuration changes:

```sh
cd src/rust
bin/format --update
```
(This can also be useful when rebasing any work-in-flight on top of a
non-trivial formatting change.)

[blog]: https://blog.rust-lang.org/
[env]: https://rust-lang.github.io/rustup/environment-variables.html
[history]: https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu.html
[options]: https://rust-lang.github.io/rustfmt/
[rustfmt_toml]: .rustfmt.toml

# Python Chores

These chores pertain to our Python codebase.

## Update graplctl's Docker Swarm region-to-AMI mapping

We maintain a static mapping called `REGION_TO_AMI_ID` in
[docker_swarm_ops.py](./src/python/graplctl/grapctl/docker_swarm_ops.py)
between AWS Region and Amazon Machine Image (AMI) IDs for the Docker
Swarm clusters managed via `graplctl`. This mapping should be updated
periodically as new AMIs become available. Instructions for updating
the mapping are provided in a source code comment.
