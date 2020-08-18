import boto3

from mypy_boto3 import sqs

from chalice import Chalice

app = Chalice(app_name="analyzer-deployer")


def _create_queue(queue_name: str):
    client: sqs.Client = boto3.resource("sqs")
    client.create_queue()
    pass


@app.route("/")
def index():
    return {"hello": "world"}


# The view function above will return {"hello": "world"}
# whenever you make an HTTP GET request to '/'.
#
# Here are a few more examples:
#
# @app.route('/hello/{name}')
# def hello_name(name):
#    # '/hello/james' -> {"hello": "james"}
#    return {'hello': name}
#
# @app.route('/users', methods=['POST'])
# def create_user():
#     # This is the JSON body the user sent in their POST request.
#     user_as_json = app.current_request.json_body
#     # We'll echo the json body back to the user in a 'user' key.
#     return {'user': user_as_json}
#
# See the README documentation for more examples.
#
