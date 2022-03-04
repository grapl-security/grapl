## Organization Management

### When making changes to the organization management service:

- Table definitions can be found and altered in the migration file in the `migrations` folder
- We can apply table changes, generate the `sql-data.json`  file by running our rebuild script `./rebuild.sh` in the `organization-management` folder. This script creates and populates a local database in a docker container.

### The path to the proto files are:
`./src/proto/graplinc/grapl/api/organization_management/v1beta1/organization_management.proto`

### Changes to the above `.proto` file should also be applied to:
`src/rust/rust-proto/src/organization_management.rs`


