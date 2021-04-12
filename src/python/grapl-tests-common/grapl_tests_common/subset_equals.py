from typing import Any, List, Mapping

 
def _dict_subset_equals(larger: Mapping, smaller: Mapping, path: str) -> None:
    for k in smaller.keys():
        new_path = f"{path}[{k}]"
        if k not in larger:
            raise SubsetEqualsException("No key {k} in larger", larger, smaller, new_path)
        _subset_equals(larger=larger[k], smaller=smaller[k], path=new_path)

def _list_subset_equals(larger: List, smaller: List, path: str) -> None:
    """
    Example: [1, 2], [2] => true, the smaller is a subset of the larger.
    We do not care about order.
    This is N^2 and I don't care.
    """
    for idx, item in enumerate(smaller):
        new_path = f"{path}[{idx}]"
        # try to find a match in the larger set
        found_match = False
        for larger_item in larger:
            try:
                _subset_equals(larger=larger_item, smaller=item, path=new_path)
            except SubsetEqualsException as e:
                pass
            else:
                # success, found a match
                found_match = True
                break

        if found_match:
            continue # on to the next item in the smaller-set
        else:
            raise SubsetEqualsException("Couldn't find a match for item in larger.", larger=larger, smaller=item, path=new_path)

def _primitive_equals(larger: object, smaller: object, path: str) -> None:
    primitives = (int, str, bool, float)
    if any((isinstance(larger, p) and isinstance(smaller, p) for p in primitives)):
        if larger != smaller:
            raise SubsetEqualsException("Not equal:", larger, smaller, path)
    else:
        raise SubsetEqualsException("Don't know how to subset-compare this type", larger, smaller, path)

class SubsetEqualsException(AssertionError):
    def __init__(self, message: str, larger: Any, smaller: Any, path: str) -> None:
        super(SubsetEqualsException, self).__init__(
            f"{message}\n\n{path}\n\n==Larger==\n{larger}\n\n==Smaller==\n{smaller}"
        )


def _subset_equals(larger: object, smaller: object, path: str = "") -> None:
    if larger is smaller:
        pass # we good
    elif isinstance(larger, List) and isinstance(smaller, List):
        _list_subset_equals(larger, smaller, path)
    elif isinstance(larger, Mapping) and isinstance(smaller, Mapping):
        _dict_subset_equals(larger, smaller, path)
    else:
        _primitive_equals(larger, smaller, path)

def subset_equals(*, larger: object, smaller: object) -> None:
    """
    in fancy terms,
    Larger = superset
    Smaller = subset
    """
    path = "root_object"
    try:
        _subset_equals(larger=larger, smaller=smaller, path=path)
    except SubsetEqualsException as e:
        raise SubsetEqualsException("Couldn't find a subset", larger, smaller, path) from e
