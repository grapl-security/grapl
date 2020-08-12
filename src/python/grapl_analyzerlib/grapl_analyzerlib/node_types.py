from copy import deepcopy
from enum import Enum

from typing import Type


class PropPrimitive(Enum):
    Int = 1
    Str = 2
    Bool = 3


class PropType(object):
    def __init__(self, primitive: PropPrimitive, is_set: bool):
        self.primitive = primitive
        self.is_set = is_set


class EdgeRelationship(Enum):
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
