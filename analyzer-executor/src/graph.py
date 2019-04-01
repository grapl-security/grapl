from abc import abstractmethod


class Not(object):
    def __init__(self, value):
        self.value = value


def strip_outer_curly(s):
    s = s.replace("{", "", 1)
    return s[::-1].replace("}", "", 1)[::-1]


def batch_queries(queries):
    stripped = [strip_outer_curly(query) for query in queries]
    enumerated = [stripped_q.replace("q0", "q{}".format(i)) for i, stripped_q in enumerate(stripped)]
    return """
    {{
    {}
    }}
    """.format("\n".join(enumerated))


def base_query(root_key, root_filter, fields):
    return """
        {{
        q0(func: has({root_key})) @cascade
         {root_filter}
         
        {{
          uid,
          {fields}
        }}
        }}
    """.format(
            root_key=root_key,
            root_filter=root_filter,
            fields=fields,
    )


class Filter(object):
    def __init__(self, predicate, value):
        self.predicate = predicate
        self.value = value

    @abstractmethod
    def build(self):
        pass


class Has(Filter):
    def build(self):
        return "has({})".format(self.predicate)


class Contains(Filter):
    def build(self):
        if isinstance(self.value, Not):
            return 'NOT regexp({}, /{}/)'.format(self.predicate, self.value.value)
        return 'regexp({}, /{}/)'.format(self.predicate, self.value)


class Eq(Filter):
    def build(self):
        if isinstance(self.value, Not):
            return 'NOT eq({}, "{}")'.format(self.predicate, self.value.value)
        return 'eq({}, "{}")'.format(self.predicate, self.value)



class Process(object):
    def __init__(self):
        self.image_name = None
        self.child = None
        self.node_key = None

    def with_image_name(self, contains=None, eq=None):
        if contains:
            self.image_name = Contains("image_name", contains)
        elif eq:
            self.image_name = Eq("image_name", eq)
        else:
            self.image_name = Has("image_name", None)

        return self

    def with_node_key(self, eq=None):
        if eq:
            self.node_key = Eq("node_key", eq)
        else:
            self.node_key = Has("node_key", None)

        return self

    def with_child(self, child):
        self.child = child
        return self

    def to_query(self):
        # print(self.get_predicate_filters())
        # print(self.get_child_filters())
        query = base_query(
           "pid",
            self.get_predicate_filters(),
            self.get_child_filters()
        )
        return query

    def get_predicate_filters(self):
        fields = [self.image_name, self.node_key]
        fields = [field.build() for field in fields if field]
        fields = " AND ".join(fields)

        field_query = """
            @filter(
                {}
            )
        """.format(fields)

        return field_query

    def get_child_filters(self):
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
            self.child.get_child_filters()
        )


class File(object):
    def __init__(self):
        self.path = None
        self.node_key = None

    def with_path(self, eq=None, contains=None):
        if eq:
            self.path = Eq('path', eq)
        elif contains:
            self.path = Contains('path', contains)
        else:
            self.path = Has('path', None)
        return self

    def with_node_key(self, eq):
        self.node_key = Eq('node_key', eq)
        return self


if __name__ == '__main__':
    node_key = "random uuid"
    child = Process()\
        .with_image_name(contains="svchost.exe")\
        .with_node_key(eq=node_key)

    parent = Process()\
        .with_image_name(contains=Not("services.exe"))
    parent.with_child(child)

    file = File()\
        .with_path(contains="C:\\Windows\\")

    query = parent.to_query()
    print(query)