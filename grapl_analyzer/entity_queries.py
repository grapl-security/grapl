import json

from typing import TypeVar, List

from abc import abstractmethod

from grapl_analyzer.entity_views import FileView, ProcessView


class Not(object):
    def __init__(self, value) -> None:
        self.value = value


def strip_outer_curly(s) -> str:
    s = s.replace("{", "", 1)
    return s[::-1].replace("}", "", 1)[::-1]


def batch_queries(queries) -> str:
    stripped = [strip_outer_curly(query_str) for query_str in queries]

    enumerated = [
        stripped_q.replace("q0", "q{}".format(i))
        for i, stripped_q in enumerate(stripped)
    ]
    return """
    {{
    {}
    }}
    """.format(
        "\n".join(enumerated)
    )


def base_query(root_key, root_filter, fields, first=None, count=True) -> str:
    count_var = ""
    if count:
        count_var = "u as"

    query_name = "q0"
    if count:
        query_name = "var"

    first_filter = ""
    if first:
        first_filter = ", first: {}".format(first)

    root_query = """
        {{
        {query_name}(func: has({root_key}) {first_filter}) @cascade
         {root_filter}
         
        {{
          {count_var} uid,
          {fields}
        }}
        }}
    """.format(
        root_key=root_key, root_filter=root_filter, fields=fields,
        count_var=count_var, query_name=query_name,
        first_filter=first_filter
    )

    if count:
        root_query += """
        result(func: uid(u)) {{
                  count(uid) as q0
                }}
        """
    return root_query


class Filter(object):
    def __init__(self, predicate, value) -> None:
        self.predicate = predicate
        self.value = value

    @abstractmethod
    def build(self) -> str:
        pass


class Has(Filter):
    def build(self) -> str:
        return "has({})".format(self.predicate)


class Contains(Filter):
    def build(self) -> str:
        if isinstance(self.value, Not):
            return "NOT regexp({}, /{}/)".format(self.predicate, self.value.value)
        return "regexp({}, /{}/)".format(self.predicate, self.value)


class Eq(Filter):
    def build(self) -> str:
        if isinstance(self.value, Not):
            return 'NOT eq({}, "{}")'.format(self.predicate, self.value.value)
        return 'eq({}, "{}")'.format(self.predicate, self.value)


P = TypeVar('P', bound='ProcessQuery')
F = TypeVar('F', bound='FileQuery')


class ProcessQuery(object):
    def __init__(self) -> None:
        self.image_name = None
        self.child = None
        self.node_key = None
        self.uid = None
        self.file_query = None
        self.create_time = None
        self.deleted_files = None  # type: List[FileView]
        self.first = None

    def only_first(self, count) -> P:
        self.first = count
        return self

    def with_image_name(self, contains=None, eq=None) -> P:
        if contains:
            self.image_name = Contains("image_name", contains)
        elif eq:
            self.image_name = Eq("image_name", eq)
        else:
            self.image_name = Has("image_name", None)

        return self

    def with_deleted_files(self, file):
        self.deleted_files = file
        return self

    def with_node_key(self, eq=None) -> P:
        if eq:
            self.node_key = Eq("node_key", eq)
        else:
            self.node_key = Has("node_key", None)

        return self

    def with_uid(self, eq=None) -> P:
        if eq:
            self.uid = Eq("uid", eq)
        else:
            self.uid = Has("uid", None)

        return self

    def with_bin_file(self, file_query) -> P:
        self.file_query = file_query
        return self

    def with_create_time(self, eq=None, before=None, after=None) -> P:
        if eq:
            self.create_time = Eq("create_time", eq)
        else:
            self.create_time = Has("create_time", None)

        return self

    def with_child(self, child) -> P:
        self.child = child
        return self

    def get_count(self, dgraph_client, only_count=True) -> P:
        query_str = base_query(
            "pid", self.get_predicate_filters(), self.get_child_filters(),
            count=True,
            first=self.first
        )
        return json.loads(dgraph_client.txn(read_only=True).query(query_str).json)[
            "count"
        ]

    def query(self, dgraph_client) -> List[P]:
        query_str = self.to_query()
        raw_views = json.loads(
            dgraph_client
            .txn(read_only=True)
            .query(query_str)
            .json
        )['q0']

        processes = []

        for raw_process in raw_views:
            processes.append(ProcessView.from_dict(dgraph_client, raw_process))

        return processes

    def to_query(self) -> str:
        # print(self.get_predicate_filters())
        # print(self.get_child_filters())
        query_str = base_query(
            "pid", self.get_predicate_filters(), self.get_child_filters(),
            first=self.first
        )
        return query_str

    def get_predicate_filters(self) -> str:
        fields = [self.image_name, self.node_key]
        fields = [field.build() for field in fields if field]
        fields = " AND ".join(fields)

        field_query = """
            @filter(
                {}
            )
        """.format(
            fields
        )

        return field_query

    def get_child_filters(self) -> str:
        if not self.child:
            return ""

        return """
            children {} {{
                uid,
                {}
                {}
            }}
        """.format(
            self.child.get_predicate_filters(),
            "image_name, node_key",
            self.child.get_child_filters(),
        )


class FileQuery(object):
    def __init__(self) -> None:
        self.path = None
        self.node_key = None

    def with_path(self, eq=None, contains=None) -> F:
        if eq:
            self.path = Eq("path", eq)
        elif contains:
            self.path = Contains("path", contains)
        else:
            self.path = Has("path", None)
        return self

    def with_node_key(self, eq) -> F:
        self.node_key = Eq("node_key", eq)
        return self


if __name__ == "__main__":
    node_key = "random uuid"
    child = (
        ProcessQuery()
        .with_image_name(contains="svchost.exe")
        .with_node_key(eq=node_key)
    )

    parent = ProcessQuery().with_image_name(contains=Not("services.exe"))
    parent.with_child(child)

    file = FileQuery().with_path(contains="C:\\Windows\\")

    query = parent.to_query()
    print(query)
