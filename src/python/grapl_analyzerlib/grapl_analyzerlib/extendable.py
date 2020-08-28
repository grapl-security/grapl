import abc

from typing import Generic, Type, TypeVar


class Extendable(abc.ABC):
    @classmethod
    def extend_self(cls, *types):
        """
        extend_self is a method that performs some monkeypatching to allow combinations of types.

        :param types: A var arg of types, all of which must implement the Extendable interface
        :return: Returns a new class, which inherits from 'cls' and all passed in types, the returned class
            will also have all methods of all types that are not prefixed with __
        """

        return extend_self_helper(cls, *types)


def extend_self_helper(cls, *types):
    cls_name = (
        cls.__name__
    )  # this might need to have the module name stripped, i forget
    for t in types:
        method_list = [method for method in dir(t) if method.startswith("__") is False]
        for method in method_list:
            setattr(cls, method, getattr(t, method))
    return type(cls_name, types, {})
