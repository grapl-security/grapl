import os
import sys

import requests


def main(plugins_folder):
    print(f'deploying {plugins_folder}')

    paths = []
    for subdir, dirs, files in os.walk(plugins_folder):
        for file in files:
            if file.endswith('.pyc'):
                continue
            paths.append(os.path.abspath(os.path.join(subdir, file)))

    plugin_dict = {}
    for path in paths:
        with open(path, 'r') as f:
            clean_path = str(path).split("model_plugins/")[-1]
            plugin_dict[clean_path] = f.read()

    headers = {'Content-type': 'application/json'}

    print(requests.post("http://localhost:8123/deploy", json={'plugins': plugin_dict}, headers=headers))


if __name__ == '__main__':
    main(sys.argv[1])

