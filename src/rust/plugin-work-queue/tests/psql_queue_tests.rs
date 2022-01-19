#![cfg(feature = "integration")]

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use plugin_work_queue::{
        psql_queue::{
            get_generator_status,
            get_generator_status_by_plugin_id,
            ExecutionId,
            PsqlQueue,
            Status,
        },
        PluginWorkQueueServiceConfig,
    };
    use sqlx::{
        postgres::PgPoolOptions,
        Pool,
        Postgres,
    };
    use structopt::StructOpt;

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

    async fn make_pool(
        service_config: PluginWorkQueueServiceConfig,
    ) -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
        let postgres_address = format!(
            "postgresql://{}:{}@{}:{}",
            service_config.plugin_work_queue_db_username,
            service_config.plugin_work_queue_db_password,
            service_config.plugin_work_queue_db_hostname,
            service_config.plugin_work_queue_db_port,
        );

        let pool = PgPoolOptions::new().connect(&postgres_address).await?;
        Ok(pool)
    }

    #[test_log::test(tokio::test)]
    async fn test_get_and_success() -> Result<(), Box<dyn std::error::Error>> {
        let config = PluginWorkQueueServiceConfig::from_args();
        tracing::info!(message = "test_get_and_success", config=?config);
        let pool = make_pool(config).await?;
        sqlx::migrate!().run(&pool).await?;
        let queue = PsqlQueue { pool };

        // Ensure one message is queued
        let plugin_id = uuid::Uuid::new_v4();
        queue
            .put_generator_message(
                plugin_id.clone(),
                b"some-message".to_vec(),
                uuid::Uuid::new_v4(),
            )
            .await?;

        let status = get_generator_status_by_plugin_id(&queue.pool, &plugin_id).await?;
        assert_eq!(status, Status::Enqueued);

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
        let config = PluginWorkQueueServiceConfig::from_args();
        tracing::info!(message = "test_get_and_failure", config=?config);
        let pool = make_pool(config).await?;
        sqlx::migrate!().run(&pool).await?;

        let queue = PsqlQueue { pool };

        // Ensure one message is queued
        let plugin_id = uuid::Uuid::new_v4();
        queue
            .put_generator_message(
                plugin_id.clone(),
                b"some-message".to_vec(),
                uuid::Uuid::new_v4(),
            )
            .await?;

        let status = get_generator_status_by_plugin_id(&queue.pool, &plugin_id).await?;
        assert_eq!(status, Status::Enqueued);

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
