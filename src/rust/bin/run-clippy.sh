#!/bin/bash

set -euo pipefail

# Until such time as we can specify clippy lints in a real file (like rustfmt),
# we'll use this script to encapsulate how we want to run it. (Clippy does have
# a configuration file, but it seems to currently be only for specifying
# parameters for specific lints, and not for specifying what level the lints
# should be checked at (warn, allow, etc.)).
#
# This script is intended to be run both locally and in CI; it is the source of
# truth for how to run clippy for this project.
#######################################################################

# Default to human-readable format (i.e., what you get from running clippy
# normally).
#
# If you wish to override this to get another format (e.g., you need
# machine-readable JSON output for integration with your editor), pass that
# format name as an argument to the script.
#
# Acceptable values correspond to the `--message-format` option (run `cargo help
# check` for details). "json" is a common choice.
format="${1:-human}"

# For detailed information on what each lint does, see
# https://rust-lang.github.io/rust-clippy/master/index.html

# NOTE: the current batch of lints is what our codebase currently
# *violates*. In time, we should clean these violations up and move to
# deny most, if not all, of these lints.
cargo clippy \
      --all-targets \
      --message-format="${format}" \
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
