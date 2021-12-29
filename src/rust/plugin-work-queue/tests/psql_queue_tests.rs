#![cfg(feature = "integration")]

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use plugin_work_queue::psql_queue::{
        get_generator_status,
        ExecutionId,
        PsqlQueue,
        Status,
    };
    use sqlx::postgres::PgPoolOptions;

    #[derive(Debug)]
    pub struct ExecutionRequest {
        execution_key: ExecutionId,
        plugin_id: uuid::Uuid,
        pipeline_message: Vec<u8>,
        status: Status,
        creation_time: DateTime<Utc>,
        last_updated: DateTime<Utc>,
        visible_after: Option<DateTime<Utc>>,
        try_count: i32,
    }

    #[tokio::test]
    async fn test_migrate() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://postgres:password@localhost:5432/postgres")
            .await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_and_success() -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(message = "test_get_and_success");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://postgres:password@localhost:5432/postgres")
            .await?;
        sqlx::migrate!().run(&pool).await?;
        let queue = PsqlQueue { pool };

        // Ensure one message is queued
        queue
            .put_generator_message(
                uuid::Uuid::new_v4(),
                b"some-message".to_vec(),
                uuid::Uuid::new_v4(),
            )
            .await?;

        // Retrieve a message
        let msg = queue
            .get_generator_message()
            .await?
            .expect("No valid message");
        let execution_key = msg.request.execution_key;
        tracing::info!(message="Received message", execution_key=?msg.request.execution_key);
        // Acknowledge the message
        queue
            .ack_generator(execution_key, Status::Processed)
            .await?;

        // The request should be marked as processed
        let status = get_generator_status(&queue.pool, &execution_key).await?;
        assert_eq!(status, Status::Processed);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_and_failure() -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(message = "test_get_and_failure");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://postgres:password@localhost:5432/postgres")
            .await?;

        let queue = PsqlQueue { pool };

        // Ensure one message is queued
        queue
            .put_generator_message(
                uuid::Uuid::new_v4(),
                b"some-message".to_vec(),
                uuid::Uuid::new_v4(),
            )
            .await?;

        // Retrieve a message
        let msg = queue
            .get_generator_message()
            .await?
            .expect("No valid message");
        let execution_key = msg.request.execution_key;
        // Acknowledge the message
        tracing::info!(message="Received message", execution_key=?msg.request.execution_key);
        queue.ack_generator(execution_key, Status::Failed).await?;
        let status = get_generator_status(&queue.pool, &execution_key).await?;
        assert_eq!(status, Status::Failed);

        Ok(())
    }
}
