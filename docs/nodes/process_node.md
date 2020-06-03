## Process Node

### Schema
| Predicate           | Type          | Description  |
| ------------------- |:-------------:| ------------:|
| node_key | string | A unique identifier for this node.
| asset_id | string | A unique identifier for an asset.
| image_name | string | The name of the binary that was loaded for this process.
| process_name | string | The name of the process.
| arguments | string | The arguments, as passed into the process.
| process_id | int | The process id for this process.
| created_timestamp | int | Milliseconds since epoch Unix to time of the process creation.
| terminate_time | int | Milliseconds since epoch Unix to time of the process termination.
| children | [[Process](/nodes/process_node/)] | Child processes of this process.
| bin_file | File | The file that was executed to create this process.
| created_files | [[File](/nodes/file_node/)] | Files created by this process.
| deleted_files | [[File](/nodes/file_node/)] | Files deleted by this process.
| read_files | [[File](/nodes/file_node/)] | Files read by this process.
| wrote_files | [[File](/nodes/file_node/)] | Files written by this process.
| created_connections | [[ProcessOutboundConnection](/nodes/process_outbound_connection_node)] | Outbound Connections created by this process.
| inbound_connections | [[ProcessInboundConnection](/nodes/process_inbound_connection_node)] | Inbbound Connections created by this process.


## ProcessQuery

###### with_node_key
```python
def with_node_key(
    self,
    eq: str,
) -> ProcessQuery:
    pass
```


###### with_asset_id
```python
def with_asset_id(
    self,
    eq: Optional[str] = None,
    contains: Optional[str] = None,
    ends_with: Optional[str] = None,
    starts_with: Optional[str] = None,
    regexp: Optional[str] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> ProcessQuery:
    pass
```

###### with_image_name
```python
def with_image_name(
    self,
    eq: Optional[str] = None,
    contains: Optional[str] = None,
    ends_with: Optional[str] = None,
    starts_with: Optional[str] = None,
    regexp: Optional[str] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> ProcessQuery:
    pass
```

###### with_process_name
```python
def with_process_name(
    self,
    eq: Optional[str] = None,
    contains: Optional[str] = None,
    ends_with: Optional[str] = None,
    starts_with: Optional[str] = None,
    regexp: Optional[str] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> ProcessQuery:
    pass
```

###### with_arguments
```python
def with_arguments(
    self,
    eq: Optional[str] = None,
    contains: Optional[str] = None,
    ends_with: Optional[str] = None,
    starts_with: Optional[str] = None,
    regexp: Optional[str] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> ProcessQuery:
    pass
```

###### with_process_id
```python
def with_process_id(
    self,
    eq: Optional[int] = None,
    gt: Optional[int] = None,
    lt: Optional[int] = None,
) -> ProcessQuery:
    pass
```

###### with_created_timestamp
```python
def with_created_timestamp(
    self,
    eq: Optional[int] = None,
    gt: Optional[int] = None,
    lt: Optional[int] = None,
) -> ProcessQuery:
    pass
```

###### with_terminate_time

```python
def with_terminate_time(
    self,
    eq: Optional[int] = None,
    gt: Optional[int] = None,
    lt: Optional[int] = None,
) -> ProcessQuery:
    pass
```

###### with_children
```python
def with_children(
    self,
    child_query: Optional["IProcessQuery"],
) -> ProcessQuery:
    pass
```

###### with_bin_file
```python
def with_bin_file(
    self,
    bin_file_query: Optional["IFileQuery"]
) -> ProcessQuery:
    pass
```

###### with_created_files
```python
def with_created_files(
    self,
    created_files_query: Optional["IFileQuery"]
) -> ProcessQuery:
    pass
```

###### with_deleted_files
```python
def with_deleted_files(
    self,
    deleted_files_query: Optional["IFileQuery"]
) -> ProcessQuery:
    pass
```

###### with_read_files
```python
def with_read_files(
    self,
    read_files_query: Optional["IFileQuery"]
) -> ProcessQuery:
    pass
```

###### with_wrote_files
```python
def with_wrote_files(
    self,
    wrote_files_query: Optional["IFileQuery"]
    ) -> ProcessQuery:
    pass
```

###### with_created_connections
```python
def with_created_connections(
    self,
    created_connection_query: Optional[
                "IProcessOutboundConnectionQuery"
            ]
    ) -> ProcessQuery:
    pass
```

###### with_inbound_connections
```python
def with_inbound_connections(
    self,
    inbound_connection_query: Optional[
                "IProcessInboundConnectionQuery"
            ]
    ) -> ProcessQuery:
    pass
```

###### with_parent
```python
def with_parent(
    self, 
    parent_query: Optional["IProcessQuery"]
) -> ProcessQuery:
    pass
```