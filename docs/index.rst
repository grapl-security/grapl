.. _Github: https://github.com/grapl-security/grapl

Grapl
=====

Grapl is a Graph Platform for Detection and Response with a focus on helping
Detection Engineers and Incident Responders stop fighting their data and start
connecting it. Find out more on our `Github`_.

For now, our documentation primarily focuses on `grapl_analyzerlib`.
`grapl_analyzerlib` provides a Python interface for end-users to interact
with the data in Grapl.

.. note::
    Grapl's documentation is still a work in progress.

.. Defines the table of contents (toc) - mostly pointing at subdirectories.
.. toctree::
    :caption: Documentation
    :maxdepth: 1

    queryable
    analyzers/index
    setup/index
    plugins/index

.. toctree::
    :caption: Development
    :maxdepth: 1
    :glob:

    development/*

Queries and Views
-----------------

Queries and Views are the main constructs to work with the graph.

Queries allow you to pull data from the graph that matches a structure.

Views represent an existing graph, which you can expand by pivoting off
of its edges.

Let's query for some processes with the name "svchost".

.. code-block:: python

    # TODO replace with instructions for next-gen grapl_analyzerlib


Now we can pivot around that data. Let's look at the parent processes of these svchosts:

.. code-block:: python

    # TODO replace with instructions for next-gen grapl_analyzerlib


Installation
------------

Install grapl_analyzerlib by running:

``pip install --user grapl_analyzerlib``

License
-------

The project is licensed under the Apache 2.0 license.
