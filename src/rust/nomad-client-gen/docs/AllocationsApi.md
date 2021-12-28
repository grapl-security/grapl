# \AllocationsApi

All URIs are relative to *https://127.0.0.1:4646/v1*

| Method                                                   | HTTP request         | Description |
| -------------------------------------------------------- | -------------------- | ----------- |
| [**get_allocations**](AllocationsApi.md#get_allocations) | **Get** /allocations |

## get_allocations

> Vec<crate::models::AllocationListStub> get_allocations(region, namespace,
> index, wait, stale, prefix, x_nomad_token, per_page, next_token, resources,
> task_states)

### Parameters

| Name              | Type               | Description                                                                    | Required | Notes |
| ----------------- | ------------------ | ------------------------------------------------------------------------------ | -------- | ----- |
| **region**        | Option<**String**> | Filters results based on the specified region.                                 |          |
| **namespace**     | Option<**String**> | Filters results based on the specified namespace.                              |          |
| **index**         | Option<**i32**>    | If set, wait until query exceeds given index. Must be provided with WaitParam. |          |
| **wait**          | Option<**String**> | Provided with IndexParam to wait for change.                                   |          |
| **stale**         | Option<**String**> | If present, results will include stale reads.                                  |          |
| **prefix**        | Option<**String**> | Constrains results to jobs that start with the defined prefix                  |          |
| **x_nomad_token** | Option<**String**> | A Nomad ACL token.                                                             |          |
| **per_page**      | Option<**i32**>    | Maximum number of results to return.                                           |          |
| **next_token**    | Option<**String**> | Indicates where to start paging for queries that support pagination.           |          |
| **resources**     | Option<**bool**>   | Flag indicating whether to include resources in response.                      |          |
| **task_states**   | Option<**bool**>   | Flag indicating whether to include task states in response.                    |          |

### Return type

[**Vec<crate::models::AllocationListStub>**](AllocationListStub.md)

### Authorization

[X-Nomad-Token](../README.md#X-Nomad-Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#)
[[Back to API list]](../README.md#documentation-for-api-endpoints)
[[Back to Model list]](../README.md#documentation-for-models)
[[Back to README]](../README.md)
