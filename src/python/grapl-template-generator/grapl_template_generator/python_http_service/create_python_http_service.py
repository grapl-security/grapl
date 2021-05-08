from grapl_templates.python_http_service.create_python_http_service_args import CreatePythonHttpServiceArgs


# {
#     "project_name": "My New Project",
#     "project_slug": "{{ cookiecutter.project_name|lower|replace(' ', '-') }}",
#     "pkg_name": "{{ cookiecutter.project_slug|replace('-', '_') }}",
#     "pants_version": "2.5.0",
#     "pants_python_interpreter_constraints": "CPython==3.7.*",
#     "pants_black_version_constraint": "black==20.8b1",
#     "pants_isort_version_constraint": "isort==5.6.4",
#     "pants_mypy_version_constraint": "mypy==0.800",
#     "lambda_handler": "lambda_handler"
# }

class PythonHttpServiceTemplate(object):
    def __init__(self) -> None:
        ...

def create_python_http_service(args: CreatePythonHttpServiceArgs) -> None:
    return None
