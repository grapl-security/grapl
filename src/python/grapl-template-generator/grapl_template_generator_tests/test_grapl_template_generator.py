from grapl_template_generator_tests.shared import invoke_main


def test_very_basic_invoke() -> None:
    result = invoke_main(["--help",])

    assert "Create a Rust gRPC project" in result.stdout


# It'd be GREAT to add some tests that actually call stuff like
# `grapl-template-generator py-http Coolproject`
# -but- unfortunately, this tool is meant to have access to + mutate the whole
# source tree - which is _completely_ against Pants's hermetic chroots stuff.
#
# Could be achieved with a Dockerized test, though.
