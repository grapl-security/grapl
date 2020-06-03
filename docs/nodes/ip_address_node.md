## Process Node

### Schema
| Predicate           | Type          | Description  |
| ------------------- |:-------------:| ------------:|
| node_key | string | A unique identifier for this node.
| ip_address | string | The IP address that this node represents.


## ProcessQuery

###### ProcessQuery.with_node_key
```python
def with_node_key(
    self,
    eq: str,
) -> ProcessQuery:
    pass
```


### ProcessQuery.with_ip_address
```python
    def with_ip_address(
        self,
        eq: Optional["StrCmp"] = None,
        contains: Optional["StrCmp"] = None,
        ends_with: Optional["StrCmp"] = None,
    ) -> "NQ":
        pass
```


### ProcessQuery.with_first_seen_timestamp
```python
    def with_first_seen_timestamp(
        self,
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        pass
```


### ProcessQuery.with_last_seen_timestamp
```python
    def with_last_seen_timestamp(
        self,
        eq: Optional["IntCmp"] = None,
        gt: Optional["IntCmp"] = None,
        lt: Optional["IntCmp"] = None,
    ) -> "NQ":
        pass
```


### ProcessQuery.with_ip_connections
```python
    def with_ip_connections(
        self,
        ip_connections_query: Optional["IpConnectionQuery"] = None
    ) -> "NQ":
        pass
```

### ProcessQuery.with_ip_connections_from
```python
    def with_ip_connections_from(
        self,
        ip_connections_from_query: Optional["IpConnectionQuery"] = None,
    ) -> "NQ":
        pass
```

### ProcessQuery.with_bound_by
```python
    def with_bound_by(
        self,
        bound_by_query: Optional["IProcessInboundConnectionQuery"] = None,
    ) -> "NQ":
        pass
```