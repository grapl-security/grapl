# How to use Dgraph Ratel on Nomad

## First get your Alphas HTTP address
http://localhost:4646/ui/jobs/grapl-core/dgraph-alpha-0
click the allocation
find the "Host Address" for the entry with 8080. Copy this to your clipboard.


## Then get your Ratel IP
http://localhost:4646/ui/jobs/grapl-local-infra/ratel
click the allocation
find "Host Address" and click on the address. 


## Try inserting it into your Ratel instance
- Welcome to the Ratel UI! Click "Latest."
- Put your Alphas HTTP address (in your clipboard, with :8080) in "Dgraph server URL"
- No password needed