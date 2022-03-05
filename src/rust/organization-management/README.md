## Organization Management

### The path to the proto files for our Organization Management Service:

`/src/proto/graplinc/grapl/api/organization_management/v1beta1/organization_management.proto`

### Any changes made to the `.proto` file above should also be applied to:

`/src/rust/rust-proto/src/organization_management.rs`

### When making changes to the organization management service:

- Table definitions for our Organization Management tables, `users` and `organizations` 
can be found and altered in auto generated migration file in 
  `/src/organization-management/migrations` folder.

### To generate a new migration, run:
`sqlx migrate run`

### To generate a new sqlx.json file to popluate our PostgresDb in offline mode, run:
`cargo sqlx prepare -- --lib`

- We can apply table changes and generate the `sql-data.json` file by running our
  rebuild script `./rebuild.sh` in the `organization-management` folder. This
  script creates and populates a local database in a docker container by making migrations. 








