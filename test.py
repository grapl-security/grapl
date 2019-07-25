import unittest


class TestQueryBuilding(unittest.TestCase):

    def test_query_with_key(self):
        from grapl_analyzerlib.entities import ProcessQuery
        from grapl_analyzerlib.entities import FileQuery
        droppers = (
            ProcessQuery()
            .with_bin_file(
                FileQuery()
                .with_creator(
                    ProcessQuery()
                )
            )
        )

        from grapl_analyzerlib.querying import _get_queries
        droppers = _get_queries(droppers, node_key="fooNodeKey")

        self.assertEqual(droppers.count("Binding0"), 2)
        self.assertEqual(droppers.count("Binding1"), 2)
        self.assertEqual(droppers.count("Binding2"), 2)


if __name__ == "__main__":
    unittest.main()
