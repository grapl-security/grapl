Welcome to Grapl-Analyzerlib's documentation!
=============================================

grapl_analyzerlib provides a Python interface for interacting with the
data in Grapl.

Grapl's documentation is still a work in progress.

.. Defines the table of contents (toc) - mostly pointing at subdirectories.
.. toctree::
    :maxdepth: 2
    :caption: Grapl Documentation

    nodes/index
    queryable
    analyzers/index
    setup/index
    plugins/index

Queries and Views
-----------------

Queries and Views are the main constructs to work with the graph.

Queries allow you to pull data from the graph that matches a structure.

Views represent an existing graph, which you can expand by pivoting off
of its edges.

Let's query for some processes with the name "svchost".

.. code-block:: python

    from grapl_analyzerlib.prelude import *

    # Create a client to talk to Grapl
    mclient = MasterGraphClient()

    svchosts = (
        ProcessQuery()
        .with_process_name(eq="svchost.exe")
        .query(mclient)  # Execute the query
    )  # type: List[ProcessView]


Now we can pivot around that data. Let's look at the parent processes of these svchosts:

.. code-block:: python

    for svchost in svchosts:
        if svchost.get_parent():
            print(svchost.parent.get_process_name())


Installation
------------

Install grapl_analyzerlib by running:

``pip install --user grapl_analyzerlib``

License
-------

The project is licensed under the Apache 2.0 license.
