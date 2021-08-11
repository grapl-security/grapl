# Local UI Development Workflow

The following describes a basic workflow for developing the Grapl
frontend locally. The commands should be run from the repository root.

---

Run `make up`; this will just be going in a terminal in the
background. If you need to tear this all down and start over at any
point, make sure you run `make down` before running make up again.

Run `make local-ux-upload`. This will will do a `yarn` build (if
necessary) for the engagement view and then upload it to the
appropriate S3 bucket in the local Grapl running under
`docker-compose`. Do this as many times as you like.

Run `make local-graplctl-setup`. This will upload the local analyzers
and the sysmon dataset to the local Grapl instance. Do this as many
times as you like.

The UI should be available at http://localhost:1234/index.html
