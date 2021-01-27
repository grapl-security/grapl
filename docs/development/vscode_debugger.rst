Visual Studio Code Debugger
===========================

Python debugger
---------------

Setup VSCode
~~~~~~~~~~~~
Add the following as a ``launch.json`` debug configuration in VSCode.
You'll want a different configuration for each service you want to debug; in this case,
we're debugging the ``grapl_e2e_tests``.
Each service's configuration should likely have a different path-mapping and a different port.

.. code-block:: js

    {
        "version": "0.2.0",
        "configurations": [
            {
                "name": "E2E tests debug",
                "type": "python",
                "request": "attach",
                "connect": {
                    "host": "127.0.0.1",
                    "port": 8400
                },
                // Also debug library code, like grapl-tests-common
                "justMyCode": false,
                "pathMappings": [
                    {
                        "localRoot": "${workspaceFolder}/src/python/grapl_e2e_tests",
                        "remoteRoot": "/home/grapl/grapl_e2e_tests"
                    },
                    {
                        "localRoot": "${workspaceFolder}/src/python/grapl-tests-common/grapl_tests_common",
                        "remoteRoot": "../venv/lib/python3.7/site-packages/grapl_tests_common"
                    },
                    {
                        "localRoot": "${workspaceFolder}/src/python/grapl_analyzerlib/grapl_analyzerlib",
                        "remoteRoot": "../venv/lib/python3.7/site-packages/grapl_analyzerlib"
                    }
                ]
            }
        ]
    }

Run the tests
~~~~~~~~~~~~~
Run the tests with the following. (Again, this example is strictly about the E2E Tests.)

.. code-block:: bash

    DEBUG_SERVICES=grapl_e2e_tests make test-e2e

You'll see the test start up and output the following:

.. code-block::

    >> Debugpy listening for client at <something>

At this point, start the debug configuration in your VSCode. The Python code should start moving along.
If you need to debug more than on service, ``DEBUG_SERVICES`` can be fed a comma-separated-list of service names.

Enable python debugging for another service
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
You can see all of these steps in action in `this pull request <https://github.com/grapl-security/grapl/pull/371/files>`_.

- Add the service name, and an unused port in the 84xx range, to SERVICE_TO_PORT in ``vsc_debugger.py``.
- Forward that port, for that service, in ``docker-compose.yml``
- Call ``wait_for_vsc_debugger("name_of_service_goes_here")`` in the main entrypoint for the service. 
  (Don't worry, it won't trigger the debugger unless you declare ``DEBUG_SERVICES``.)
- Add a new ``launch.json`` debug configuration for that port.
- Finally - run your `make test-e2e` command with ``DEBUG_SERVICES=name_of_service_goes_here``.


Rust debugger
-------------
We haven't invested in this yet!