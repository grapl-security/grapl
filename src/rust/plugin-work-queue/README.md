The plugin-work-queue provides a SQS-like queue abstraction over Postgresql.


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