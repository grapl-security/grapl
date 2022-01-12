The plugin-work-queue provides a SQS-like queue abstraction over Postgresql.


## Algorithm

Every row in the table represents a job to be processed. Jobs hold state about their processing history.

A job's state can be primarily determined by a combination of its `current_status` and `visible_after` values. 

Its `current_status` can be:
`enqueued` - meaning the value is either currently executing or may execute in the future
`success` - meaning the value has been successfully processed
`failed` - meaning the value has been processed unsuccessfully in a persistent manner eg: an invalid job

The `visible_after` is the time after which a job may be executed. When a job is acquired the `visible_after` is immediately
updated to the CURRENT_TIMESTAMP + 10 seconds. This value is arbitrary and may change in the future. No other worker
will acquire that job until `visible_after <= CURRENT_TIMESTAMP`.

Jobs "age out" after 1 day, meaning that even if they are in the `enqueued` state and are "visible" they will not be
acquired. Jobs that are aged out are removed after one month.

Currently we track how many times a job has been executed but we do not impose a limit. That would be a good next step.

The query for processing a message is:

1. Find the oldest message that
    1. Has not aged out (1 day)
    2. Is not currently being locked by another request
    3. Is 'enqueued'
    4. Is visible ie: the `visible_after` is <= CURRENT_TIMESTAMP
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

### Visibility Timeout
The visibility timeout is a duration in which a message will not be picked up again by another worker.

Right now we use a static visibility timeout. It likely makes more sense to increment that timeout somehow,
and to have the base timeout be based on a client provided value.


## Hardcoded Values and Next Steps
Right now we have some hardcoded values that, in the future, we can and should make dynamic.

1. The visibility timeout is 10 seconds. In the future this should be dynamic.
2. The retention window for which a message can be executed is 1 day. This just seemed like a reasonable default.

We also don't use the retry_count or have a cap on number of retries we'll execute. That should be an immediate next step.