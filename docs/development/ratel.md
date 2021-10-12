# How to use Dgraph Ratel on Nomad

## First get your Alphas HTTP address

- Navigate to http://localhost:4646/ui/jobs/grapl-core/dgraph-alpha-0
- Click the allocation
- Find the "Host Address" for the entry "dgraph-alpha-port".
- Copy this to your clipboard.

## Then get your Ratel IP

- Navigate to http://localhost:4646/ui/jobs/grapl-local-infra/ratel
- Click the allocation
- Find the single "Host Address" and click on the address.

## Try inserting it into your Ratel instance

- Welcome to the Ratel UI! Click "Latest."
- Put your Alphas HTTP address (in your clipboard) in "Dgraph server URL"
- No password needed

## Try out a query!

This will return all lenses (well, the first 1000).

```
query all()
{
    all(func: type(Lens), first: 1000, offset: 0, orderdesc: score)
    {
        lens_name,
        score,
        node_key,
        uid,
        dgraph_type: dgraph.type,
        lens_type,
        scope {
            uid,
            node_key,
            dgraph_type: dgraph.type,
        }
    }
}
```
