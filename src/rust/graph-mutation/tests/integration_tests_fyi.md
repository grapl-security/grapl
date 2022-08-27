The integration tests for this are in graph-query-service/.

You can't really test querying without mutating, and you can't really test
mutating without querying! So we arbitrarily chose one of them as the home
for tests.