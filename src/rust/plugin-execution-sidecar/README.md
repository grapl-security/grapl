# plugin-executor

This is a home for shared code between an analyzer-executor and a generator
executor, as well as an entry-point for those two services.

The gist of a generator-executor is this:

```
while true {
    - grab generator work from plugin-work-queue
    - send that work to a given plugin binary process over Consul Connect
    - receive the result back
    - put that result on a Kafka topic for `node-identifier` to read
    - ack to plugin-work-queue that the work has been completed
}
```

The gist of an analyzer-executor is basically the same, except how we grab
analyzer work and which Kafka topic we put the result on.
