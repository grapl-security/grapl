python_tests(
    # This test needs work because if we're filtering by Pytest tags
    # (rather than Pants tags), the test has to be parsed first during
    # the collection phase. The way the library is coded, however,
    # requires that a DEPLOYMENT_NAME environment variable is set;
    # otherwise the whole thing crashes.
    tags=["needs_work"],
    sources=["test_engagement_edge.py"]
)
