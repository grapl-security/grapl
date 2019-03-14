from abc import abstractmethod

base_process_query = """
    q1(func: has(pid)) @cascade
     {root_filter}
     
    {{
      uid,
      {fields}
      {children}
    }}
"""


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
            return 'NOT alloftext({}, "{}")'.format(self.predicate, self.value.value)
        return 'alloftext({}, "{}")'.format(self.predicate, self.value)


class Eq(Filter):
    def build(self):
        if isinstance(self.value, Not):
            return 'NOT eq({}, "{}")'.format(self.predicate, self.value.value)
        return 'eq({}, "{}")'.format(self.predicate, self.value)


class Not(object):
    def __init__(self, value):
        self.value = value


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
            self.node_key = Eq("node_key", node_key)
        else:
            self.node_key = Has("node_key", node_key)

        return self

    def with_child(self, child):
        self.child = child

    def to_query(self):
        # print(self.get_predicate_filters())
        # print(self.get_child_filters())
        return base_process_query.format(
            root_filter = self.get_predicate_filters(),
            fields="",
            children = self.get_child_filters()
        )

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

        filter = """
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

        return filter


if __name__ == '__main__':
    node_key = "random uuid"
    child = Process()\
        .with_image_name(contains="svchost.exe")\
        .with_node_key(eq=node_key)

    parent = Process()\
        .with_image_name(contains=Not("services.exe"))
    parent.with_child(child)

    query = parent.to_query()
    print(query)