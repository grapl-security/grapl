from copy import deepcopy
from enum import Enum, IntEnum

from typing import Type, Optional, List


class PropPrimitive(Enum):
    Int = 1
    Str = 2
    Bool = 3


class PropType(object):
    def __init__(
        self,
        primitive: PropPrimitive,
        is_set: bool,
        index: Optional[List[str]] = None,
        upsert=False,
    ):
        self.primitive = primitive
        self.is_set = is_set
        self.index = index
        self.upsert = upsert

    def prop_index_str(self) -> str:
        if self.index and self.index != True:
            index_str = f"@index({', '.join(self.index)})"
        elif self.primitive is PropPrimitive.Str:
            index_str = "@index(exact, trigram)"
        elif self.primitive is PropPrimitive.Int:
            index_str = "@index(int)"
        elif self.primitive is PropPrimitive.Bool:
            index_str = "@index(bool)"
        else:
            raise Exception("Unreachable")

        if self.upsert:
            index_str += " @upsert"
        return index_str

    def prop_type_str(self):
        if self.primitive is PropPrimitive.Str:
            prim_str = "string"
        elif self.primitive is PropPrimitive.Int:
            prim_str = "int"
        elif self.primitive is PropPrimitive.Bool:
            prim_str = "bool"
        else:
            raise Exception("Unreachable")

        if self.is_set:
            prim_str = f"[{prim_str}]"
        return prim_str


class EdgeRelationship(IntEnum):
    OneToOne = 1
    OneToMany = 2
    ManyToMany = 3
    ManyToOne = 4

    def reverse(self) -> "EdgeRelationship":
        if self is EdgeRelationship.OneToMany:
            return EdgeRelationship.ManyToOne
        if self is EdgeRelationship.ManyToOne:
            return EdgeRelationship.OneToMany
        return self

    def is_to_many(self) -> bool:
        if self is EdgeRelationship.OneToMany:
            return True
        if self is EdgeRelationship.ManyToMany:
            return True

        return False

    def is_from_many(self) -> bool:
        if self is EdgeRelationship.ManyToOne:
            return True
        if self is EdgeRelationship.ManyToMany:
            return True

        return False

    def is_to_one(self) -> bool:
        if self is EdgeRelationship.ManyToOne:
            return True
        if self is EdgeRelationship.OneToOne:
            return True

        return False

    def is_from_one(self) -> bool:
        if self is EdgeRelationship.OneToMany:
            return True
        if self is EdgeRelationship.OneToOne:
            return True

        return False


class EdgeT(object):
    def __init__(
        self, source: Type["Schema"], dest: Type["Schema"], rel: EdgeRelationship
    ):
        self.source = source
        self.dest = dest
        self.rel = rel

    def reverse(self) -> "EdgeT":
        reversed_self = deepcopy(self)
        reversed_self.rel = reversed_self.rel.reverse()
        reversed_self.source, reversed_self.dest = (
            reversed_self.dest,
            reversed_self.source,
        )
        return reversed_self

    def is_from_one(self) -> bool:
        return self.rel.is_from_one()

    def is_to_one(self) -> bool:
        return self.rel.is_to_one()

    def is_from_many(self) -> bool:
        return self.rel.is_from_many()

    def is_to_many(self) -> bool:
        return self.rel.is_to_many()


from grapl_analyzerlib.schema import Schema
