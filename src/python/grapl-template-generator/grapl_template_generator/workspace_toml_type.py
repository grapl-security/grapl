from typing import List

from typing_extensions import TypedDict

Workspace = TypedDict("Workspace", {"members": List[str]})

WorkspaceToml = TypedDict("WorkspaceToml", {"workspace": Workspace})
