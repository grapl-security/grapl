python_library(
    name="grapl_template_generator_lib",
    sources=[
        "**/*.py",
        "!grapl-templates/**/*",
    ]
)

pex_binary(
    entry_point="main.py:app",
    zip_safe=False,
)

shell_library()