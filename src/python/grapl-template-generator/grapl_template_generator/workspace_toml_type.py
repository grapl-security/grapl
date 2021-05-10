from typing import List, TypedDict


Workspace = TypedDict('Workspace', {
    'members': List[str]
})

WorkspaceToml = TypedDict('WorkspaceToml', {
    'workspace': Workspace
})
