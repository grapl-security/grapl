#!/bin/bash

set -euo pipefail

# Encapsulates everything to do with linting Rust code.
#
# There are currently two main tools for linting Rust: the compiler
# itself, as exposed via `cargo check`, and `clippy`. Both provide
# useful checks, so it's good to provide a unified facade to both.
#
# While the compiler lints are validated when the code is compiled,
# it's useful to also expose them in a way that code editors can take
# advantage of. Here, we run both tools together, consolidating their
# output on stdout. Both produce a similar machine-readable format, so
# the end result is just one unified list of lint violations.
#
# This script is intended to be run both locally and in CI; it is the
# source of truth for how to run clippy for this project.
#######################################################################

# Default to human-readable format (i.e., what you get from running
# these tools normally).
#
# If you wish to override this to get another format (e.g., you need
# machine-readable JSON output for integration with your editor), pass
# that format name as an argument to the script.
#
# Acceptable values correspond to the `--message-format` option (run
# `cargo help check` for details; Clippy uses this, too). "json" is a
# common choice.
format="${1:-human}"

########################################################################

# Explicitly *NOT* failing the script if one of these commands fails
# (thus indicating a linting violation). If there are `cargo check`
# violations, we want to see those AND run `clippy` afterwards.
set +e

# We do, however want to keep track if something failed, so we can
# fail the overall script. This should only be set to "true" if one
# (or both!) of the linters fails.
failed="false"

# NOTE: This is configured with the `.cargo/config.toml` file.
cargo check \
      --all-targets \
      --message-format="${format}" \
      --all-features
if [ $? != 0 ]; then
    failed="true"
fi

# Until such time as we can specify Clippy lints in a real file (like rustfmt),
# we'll use this script to encapsulate how we want to run it. (Clippy does have
# a configuration file, but it seems to currently be only for specifying
# parameters for specific lints, and not for specifying what level the lints
# should be checked at (warn, allow, etc.)).
#
# For detailed information on what each lint does, see
# https://rust-lang.github.io/rust-clippy/master/index.html
#
# NOTE: the current batch of lints is what our codebase currently
# *violates*. In time, we should clean these violations up and move to
# deny most, if not all, of these lints.
cargo clippy \
      --all-targets \
      --message-format="${format}" \
      --all-features \
      -- \
      --allow clippy::char_lit_as_u8 \
      --allow clippy::clone_double_ref \
      --allow clippy::clone_on_copy \
      --allow clippy::cmp_owned \
      --allow clippy::collapsible_if \
      --allow clippy::eq_op \
      --allow clippy::expect_fun_call \
      --allow clippy::filter_next \
      --allow clippy::float_cmp \
      --allow clippy::into_iter_on_ref \
      --allow clippy::large_enum_variant \
      --allow clippy::len_zero \
      --allow clippy::let_and_return \
      --allow clippy::manual_range_contains \
      --allow clippy::needless_return \
      --allow clippy::new_ret_no_self \
      --allow clippy::op_ref \
      --allow clippy::option_as_ref_deref \
      --allow clippy::or_fun_call \
      --allow clippy::redundant_clone \
      --allow clippy::redundant_closure \
      --allow clippy::redundant_field_names \
      --allow clippy::redundant_pattern_matching \
      --allow clippy::redundant_static_lifetimes \
      --allow clippy::single_char_pattern \
      --allow clippy::single_component_path_imports \
      --allow clippy::single_match \
      --allow clippy::too_many_arguments \
      --allow clippy::unnecessary_lazy_evaluations \
      --allow clippy::unused_unit \
      --allow clippy::useless_conversion \
      --allow clippy::write_with_newline \
      --allow clippy::wrong_self_convention
if [ $? != 0 ]; then
    failed="true"
fi

# Turn exit-on-error behavior back on, on principle.
set -e

if [ "${failed}" == "true" ]; then
    >&2 echo
    >&2 echo "Linting violations found"
    exit 1
fi
