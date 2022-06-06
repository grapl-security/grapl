# generator-dispatcher

This service will read generator work from the ingress raw-logs topic, and call
into Plugin Work Queue to enqueue that work in a durable Postgres store.
