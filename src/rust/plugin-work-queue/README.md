The plugin-work-queue provides a SQS-like queue abstraction over Postgresql.

## Queue Schema
```postgresql
CREATE SCHEMA plugin_work_queue;

CREATE TYPE plugin_work_queue.status AS ENUM ('enqueued', 'failed', 'processed');

CREATE FUNCTION plugin_work_queue.megabytes(bytes integer) RETURNS integer AS
$$
BEGIN
   RETURN bytes * 1000 * 1000;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS plugin_work_queue.generator_plugin_executions
(
   execution_key    bigserial                  NOT NULL,
   tenant_id        uuid                       NOT NULL,
   plugin_id        uuid                       NOT NULL,
   pipeline_message bytea                      NOT NULL,
   current_status           plugin_work_queue.status NOT NULL,
   creation_time    timestamptz                NOT NULL,
   last_updated     timestamptz                NOT NULL,
   visible_after    timestamptz                NOT NULL DEFAULT CURRENT_TIMESTAMP,
   try_count        integer                    NOT NULL,
   CHECK (length(pipeline_message) < plugin_work_queue.megabytes(256)),
   CHECK (last_updated >= creation_time)
)
   PARTITION BY RANGE (creation_time);

CREATE TABLE IF NOT EXISTS plugin_work_queue.analyzer_plugin_executions
(
   execution_key    bigserial                  NOT NULL,
   tenant_id        uuid                       NOT NULL,
   plugin_id        uuid                       NOT NULL,
   pipeline_message bytea                      NOT NULL,
   current_status           plugin_work_queue.status NOT NULL,
   creation_time    timestamptz                NOT NULL,
   last_updated     timestamptz                NOT NULL,
   visible_after    timestamptz                NOT NULL DEFAULT CURRENT_TIMESTAMP,
   try_count        integer                    NOT NULL,
   CHECK (length(pipeline_message) < plugin_work_queue.megabytes(256)),
   CHECK (last_updated >= creation_time)
)
   PARTITION BY RANGE (creation_time);

CREATE INDEX IF NOT EXISTS execution_key_ix ON plugin_work_queue.generator_plugin_executions (execution_key);
CREATE INDEX IF NOT EXISTS creation_time_ix ON plugin_work_queue.generator_plugin_executions (creation_time);

CREATE INDEX IF NOT EXISTS execution_key_ix ON plugin_work_queue.analyzer_plugin_executions (execution_key);
CREATE INDEX IF NOT EXISTS creation_time_ix ON plugin_work_queue.analyzer_plugin_executions (creation_time);
```

`execution_key` is a `bigserial` - an auto-incrementing 64bit integer.

`plugin_id` is the uuid name of the plugin to send this execution result to.

`pipeline_message` is the raw bytes to be interpreted by the plugin. The message is limited to 256MB as an arbitrary but
reasonable upper limit.

`current_status` is an enum representing the state. 'enqueued' is the default, and a message will be set to either 'processed' or 'failed' based on its success.

`creation_time` is when the row was created.

`last_updated` is set with each update to the row

`visible_after` is essentially a visibility timeout. A `visible_after` of NULL means a message is available, which
is the default state. Every time a message is SELECT'd we update `visible_after`. See the `Visibility Timeout` section
below.

`try_count` on every receive we increment `try_count` to indicate another attempt to process this message


## Algorithm
The query for processing a message is:

1. Find the oldest message that
    1. Has not aged out (1 day)
    2. Is not currently being locked by another request
    3. Is 'enqueued'
    4. Is visible ie: the `visible_after` is null or < CURRENT_TIMESTAMP
2. Update that message
    1. Increment `try_count`
    2. Update `visible_after` to CURRENT_TIMESTAMP + an interval
3. Return the message

The message is then processed by the consumer.

If the message is successfully processed or if it fails, update the row:
1. Set the `execution_result`
2. Set the `last_updated` to CURRENT_TIMESTAMP
3. Set the `status` to `processed` or `failed` accordingly

Otherwise, if the message is not successfully processed but can be retried,
do nothing. It will be picked up again later.

Note that just because a message has the `enqueued` status does *not* mean that it will be processed. We don't
guarantee that, on failure, we'll mark the `status` as failed.

Currently we'll retry messages forever unless they're marked as `failed`. Imposing a max retry would be a good
next step.

### Visibility Timeout
Right now we use a static visibility timeout. It likely makes more sense to increment that timeout somehow,
and to have the base timeout be based on a client provided value.