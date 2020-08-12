import abc

from typing import Generic, Type, TypeVar


class Extendable(abc.ABC):
    @staticmethod
    @abc.abstractmethod
    def extend_self(*types):
        """
        A valid implementation of `extend_self` must pass a string literal
        directly as the first argument to a call to `type`, where that literal matches
        the exact name of the class name of the calling Extendable.

        The second argument must consist of *types, and the third argument an empty
        dict.

        For example, the *only* valid implementation of `extend_self` for a class `MyQuery`
        is as follows:

        ```python
        class MyQuery(Extendable[MyQuery]):
            def extend_self(*types):
                return type('MyQuery', types, {})
        ```
        :param types: A var arg of types, all of which must implement the Extendable interface
        :return: Returns a new class, which inherits from 'cls' and all passed in types
        """
        pass
