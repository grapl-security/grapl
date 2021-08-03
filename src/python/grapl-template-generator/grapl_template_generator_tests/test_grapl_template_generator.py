from grapl_template_generator_tests.shared import invoke_main


def test_invoke_python_http() -> None:
    result = invoke_main([
        "py-http",
        "Coolstuff",
    ])

    assert "Created a Python HTTP service named Coolstuff" in result.stdout
