use futures::{
    StreamExt,
    TryFutureExt,
};
use rusoto_s3::{
    AbortMultipartUploadRequest,
    CompleteMultipartUploadRequest,
    CompletedMultipartUpload,
    CompletedPart,
    CreateMultipartUploadRequest,
    S3Client,
    StreamingBody,
    UploadPartRequest,
    S3,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::CreatePluginRequest;

use super::service::PluginRegistryServiceConfig;
use crate::{
    error::{
        PluginRegistryServiceError,
        S3PutError,
    },
    exp_backoff_retry::simple_exponential_backoff_retry,
};

#[derive(Clone)]
/// A utility struct to reduce repeated fields.
pub struct S3MultipartFields {
    pub bucket: String,
    pub key: String,
    pub expected_bucket_owner: Option<String>,
}

impl From<S3MultipartFields> for CreateMultipartUploadRequest {
    fn from(fields: S3MultipartFields) -> CreateMultipartUploadRequest {
        CreateMultipartUploadRequest {
            bucket: fields.bucket,
            key: fields.key,
            expected_bucket_owner: fields.expected_bucket_owner,
            ..Default::default()
        }
    }
}

impl From<S3MultipartFields> for UploadPartRequest {
    fn from(fields: S3MultipartFields) -> UploadPartRequest {
        UploadPartRequest {
            bucket: fields.bucket,
            key: fields.key,
            expected_bucket_owner: fields.expected_bucket_owner,
            ..Default::default()
        }
    }
}

impl From<S3MultipartFields> for CompleteMultipartUploadRequest {
    fn from(fields: S3MultipartFields) -> CompleteMultipartUploadRequest {
        CompleteMultipartUploadRequest {
            bucket: fields.bucket,
            key: fields.key,
            expected_bucket_owner: fields.expected_bucket_owner,
            ..Default::default()
        }
    }
}

impl From<S3MultipartFields> for AbortMultipartUploadRequest {
    fn from(fields: S3MultipartFields) -> AbortMultipartUploadRequest {
        AbortMultipartUploadRequest {
            bucket: fields.bucket,
            key: fields.key,
            expected_bucket_owner: fields.expected_bucket_owner,
            ..Default::default()
        }
    }
}

pub struct UploadStreamMultipartOutput {
    pub stream_length: usize,
    pub completed_parts: Vec<CompletedPart>,
}

type Error = PluginRegistryServiceError;
pub async fn upload_stream_multipart_to_s3(
    request: futures::channel::mpsc::Receiver<CreatePluginRequest>,
    s3: &S3Client,
    config: &PluginRegistryServiceConfig,
    s3_multipart_fields: S3MultipartFields,
) -> Result<UploadStreamMultipartOutput, Error> {
    let put_handle = s3
        .create_multipart_upload(s3_multipart_fields.clone().into())
        .await
        .map_err(S3PutError::from)?;
    let upload_id = put_handle.upload_id.expect("upload id");
    tracing::info!(
        message = "Create Upload",
        upload_id = ?upload_id,
    );

    let upload_body_result = upload_body(
        request,
        s3,
        config,
        s3_multipart_fields.clone(),
        upload_id.clone(),
    )
    .await;
    match upload_body_result {
        Ok(out) => {
            complete_multipart_upload(
                s3,
                s3_multipart_fields,
                upload_id,
                out.completed_parts.clone(),
            )
            .await?;
            Ok(out)
        }
        Err(e) => {
            abort_multipart_upload(s3, s3_multipart_fields, upload_id).await?;
            Err(e)
        }
    }
}

/// The initial CreateMultipartUpload has happened. Now upload the entire
/// body stream.
async fn upload_body(
    request: futures::channel::mpsc::Receiver<CreatePluginRequest>,
    s3: &S3Client,
    config: &PluginRegistryServiceConfig,
    s3_multipart_fields: S3MultipartFields,
    upload_id: String,
) -> Result<UploadStreamMultipartOutput, Error> {
    let limit_bytes = config.artifact_size_limit_mb.clone() * 1024 * 1024;
    let mut stream_length = 0;

    let mut body_stream = Box::pin(request.enumerate());

    let mut completed_parts: Vec<CompletedPart> = vec![];

    // This is serial, and you're actually able to upload multiple parts
    // out-of-order in parallel; if we find this to be slow, we can
    // explore using Stream::for_each_concurrent.
    while let Some((idx, result)) = body_stream.next().await {
        // S3 PartNumber is one-indexed
        let part_number = (idx + 1) as i64;
        let bytes = match result {
            CreatePluginRequest::Chunk(c) => Ok(c),
            _ => Err(Error::StreamInputError("Expected request 1..N to be Chunk")),
        }?;
        stream_length += bytes.len();
        if stream_length > limit_bytes {
            Err(Error::StreamInputError("Input exceeds size limit"))?;
        }

        tracing::info!(message = "Uploading part", part_number = part_number,);

        let part_upload = simple_exponential_backoff_retry(|| {
            s3.upload_part(UploadPartRequest {
                body: Some(StreamingBody::from(bytes.clone())),
                upload_id: upload_id.clone(),
                part_number,
                ..s3_multipart_fields.clone().into()
            })
            .map_err(S3PutError::from)
        })
        .await?;

        completed_parts.push(CompletedPart {
            part_number: Some(part_number),
            e_tag: part_upload.e_tag,
        });
    }

    Ok(UploadStreamMultipartOutput {
        stream_length,
        completed_parts,
    })
}

async fn complete_multipart_upload(
    s3: &S3Client,
    s3_multipart_fields: S3MultipartFields,
    upload_id: String,
    completed_parts: Vec<CompletedPart>,
) -> Result<(), Error> {
    tracing::info!(
        message = "Completing multipart upload",
        upload_id = ?upload_id,
    );
    s3.complete_multipart_upload(CompleteMultipartUploadRequest {
        upload_id,
        multipart_upload: Some(CompletedMultipartUpload {
            parts: Some(completed_parts),
        }),
        ..s3_multipart_fields.clone().into()
    })
    .await
    .map_err(S3PutError::from)?;
    Ok(())
}

async fn abort_multipart_upload(
    s3: &S3Client,
    s3_multipart_fields: S3MultipartFields,
    upload_id: String,
) -> Result<(), Error> {
    tracing::info!(
        message = "Aborting multipart upload",
        upload_id = ?upload_id,
    );
    s3.abort_multipart_upload(AbortMultipartUploadRequest {
        upload_id,
        ..s3_multipart_fields.clone().into()
    })
    .await
    .map_err(S3PutError::from)?;
    Ok(())
}
