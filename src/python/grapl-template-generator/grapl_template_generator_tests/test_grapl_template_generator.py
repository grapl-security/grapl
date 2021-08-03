from grapl_template_generator_tests.shared import invoke_main


def test_very_basic_invoke() -> None:
    result = invoke_main([])

    assert "py-http" in result.stdout
    assert "rust-grpc" in result.stdout


# It'd be delightful to add some tests here that actually call stuff like
# `grapl-template-generator py-http Coolproject`
# but unfortunately, this tool is meant to have access to + mutate the whole
# source tree - which is _completely_ against Pants's hermetic chroots stuff.
