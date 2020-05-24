## File Node

### Schema
| Predicate           | Type          | Description  |
| ------------------- |:-------------:| ------------:|
| node_key | string | A unique identifier for this node.
| asset_id | string | A unique identifier for an asset.
| file_name | string | todo: description |
| file_path | string | todo: description |
| file_extension | string | todo: description |
| file_mime_type | string | todo: description |
| file_version | string | todo: description |
| file_description | string | todo: description |
| file_product | string | todo: description |
| file_company | string | todo: description |
| file_directory | string | todo: description |
| file_hard_links | string | todo: description |
| signed_status | string | todo: description |
| md5_hash | string | todo: description |
| sha1_hash | string | todo: description |
| sha256_hash | string | todo: description |
| file_size | int | todo: description |
| file_inode | int | todo: description |
| signed | bool | todo: description |


## FileQuery

###### with_asset_id
```python
def with_node_key(
    self,
    eq: str,
) -> FileQuery:
    pass
```


###### with_asset_id
```python
def with_asset_id(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
) -> FileQuery:
    pass
```

###### with_file_extension
```python
def with_file_extension(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_file_mime_type
```python
def with_file_mime_type(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_file_size
```python
def with_file_size(
    self,
    eq: Optional["IntCmp"] = None,
    gt: Optional["IntCmp"] = None,
    lt: Optional["IntCmp"] = None,
) -> FileQuery:
    pass
```

###### with_file_version
```python
def with_file_version(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_file_description
```python
def with_file_description(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_file_product
```python
def with_file_product(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_file_company
```python
def with_file_company(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_file_directory
```python
def with_file_directory(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_file_inode
```python
def with_file_inode(
    self,
    eq: Optional["IntCmp"] = None,
    gt: Optional["IntCmp"] = None,
    lt: Optional["IntCmp"] = None,
) -> FileQuery:
    pass
```

###### with_file_hard_links
```python
def with_file_hard_links(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
) -> FileQuery:
    pass
```

###### with_signed
```python
def with_signed(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
    starts_with: Optional["StrCmp"] = None,
    regexp: Optional["StrCmp"] = None,
    distance: Optional[Tuple["StrCmp", int]] = None,
) -> FileQuery:
    pass
```

###### with_signed_status
```python
def with_signed_status(
    self,
    eq: Optional["StrCmp"] = None,
    contains: Optional["StrCmp"] = None,
    ends_with: Optional["StrCmp"] = None,
) -> FileQuery:
    pass
```

###### with_md5_hash
```python
def with_md5_hash(self, eq: Optional["StrCmp"] = None) -> FileQuery:
    pass
```

###### with_sha1_hash
```python
def with_sha1_hash(self, eq: Optional["StrCmp"] = None) -> FileQuery:
    pass
```

###### with_sha256_hash
```python
def with_sha256_hash(self, eq: Optional["StrCmp"] = None) -> FileQuery:
    pass
```

###### with_spawned_from
```python
def with_spawned_from(
    self, spawned_from_query: Optional["ProcessQuery"] = None
) -> FileQuery:
    pass
```

###### with_creator
```python
def with_creator(
    self, creator_query: Optional["ProcessQuery"] = None
) -> FileQuery:
    pass
```

###### with_readers
```python
def with_readers(self, reader_query: Optional["ProcessQuery"] = None) -> FileQuery:
    pass
```