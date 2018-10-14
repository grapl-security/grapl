#![feature(nll, test, proc_macro_non_items, generators, async_await, use_extern_macros)]

extern crate aws_lambda as lambda;
extern crate openssl_probe;
extern crate base64;
#[macro_use] extern crate failure;
#[macro_use] extern crate futures_await as futures;
#[macro_use] extern crate log;
extern crate prost;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sqs;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;

use serde_json::Value;
use serde::Deserialize;

use failure::Error;
use futures::Future;
use futures::prelude::{async, async_block, await};
use futures::Stream;
use lambda::event::s3::S3Event;
use lambda::event::sns::*;
use lambda::event::sqs::{SqsEvent, SqsMessage};
use prost::Message;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3};
use rusoto_s3::S3Client;
use rusoto_sqs::{Sqs, SqsClient, GetQueueUrlRequest };
use std::rc::Rc;
use std::sync::mpsc::channel;
use tokio_core::reactor::{Core, Handle};
use std::sync::mpsc::Receiver;
use std::any::Any;
use std::thread;
use std::thread::JoinHandle;


#[async]
fn read_raw_message<S>(s3_client: Rc<S>, bucket: String, path: String) -> Result<Vec<u8>, Error>
    where S: S3 + 'static
{
    info!("Fetching data from {} {}", bucket, path);

    let object = await!(s3_client.get_object(&GetObjectRequest {
            bucket: bucket.to_owned(),
            key: path.clone(),
            ..GetObjectRequest::default()
        })).expect(&format!("get_object {} {}", bucket, path));

    let mut body = vec![];

    #[async]
        for chunk in object.body.expect("object.body") {
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

#[async]
fn read_message<M, S>(s3_client: Rc<S>, bucket: String, path: String) -> Result<M, Error>
    where M: Message + Default,
          S: S3 + 'static
{
    info!("Fetching data from {} {}", bucket, path);
    let object = await!(s3_client.get_object(&GetObjectRequest {
            bucket: bucket.to_owned(),
            key: path,
            ..GetObjectRequest::default()
        })).unwrap();

    let mut body = vec![];

    #[async]
    for chunk in object.body.unwrap() {
        body.extend_from_slice(&chunk);
    }

    Ok(M::decode(body)?)
}


pub fn get_raw_messages(event: S3Event) -> Result<Vec<Vec<u8>>, Error>
{
    // NOTE that `paths` should be assumed to be of length 1
    // https://stackoverflow.com/questions/28484421/
    // does-sqs-really-send-multiple-s3-put-object-records-per-message
    let paths: Vec<_> = event.records.into_iter().map(|record| {
        info!("retrieving s3 paths. bucket: {} key: {}",
              record.s3.clone().bucket.name.unwrap_or("missing".to_owned()),
              record.s3.clone().object.key.unwrap_or("missing".to_owned())
        );
        (record.s3.bucket.name.expect("bucket.name"),
         record.s3.object.key.expect("object.key"))
    }).collect();

    let s3_client = S3Client::simple(
        Region::UsEast1
    );

    let s3_client = Rc::new(s3_client);

    paths.into_iter()
        .map(|(bucket, object)| {
            read_raw_message(s3_client.clone(), bucket, object).wait()
        }).collect()
}


//#[async]
pub fn get_messages<M>(event: S3Event) -> Result<Vec<M>, Error>
    where M: Message + Default
{
    // NOTE that `paths` should be assumed to be of length 1
    // https://stackoverflow.com/questions/28484421/
    // does-sqs-really-send-multiple-s3-put-object-records-per-message
    let paths: Vec<_> = event.records.into_iter().map(|record| {
        (record.s3.bucket.name.unwrap(),
         record.s3.object.key.unwrap())
    }).collect();
    info!("Extracted s3 paths: {:#?}", paths);

    let s3_client = S3Client::simple(
        Region::UsEast1
    );

    let s3_client = Rc::new(s3_client);

    paths.into_iter()
        .map(|(bucket, object)| {
            read_message(s3_client.clone(), bucket, object).wait()
        }).collect()
}

#[inline(always)]
pub fn handle_raw_event<T>(f: impl Fn(Vec<u8>) -> Result<T, Error> + Clone + Send + 'static)
{
    lambda::logger::init();

    lambda::start(move |events: SqsEvent| {

        let (tx, rx) = channel();

        for message in events.records {
            let f = f.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let f = f.clone();
                let body = message.body.unwrap();
                let event: S3Event = serde_json::from_str(&body).unwrap();

                let messages = get_raw_messages(event).unwrap();

                for message in messages {
                    f(message);
                }

                tx.send((message.receipt_handle.unwrap(),
                         message.event_source_arn.unwrap())).unwrap();
//                Ok(())
            });
        }

        let sqs_client =
            SqsClient::simple(rusoto_core::region::Region::UsEast1);

        let mut queue_url: Option<String> = None;

        for (receipt_handle, arn) in rx {
            match queue_url {
                Some(ref url) => {
                    sqs_client.delete_message(
                        &rusoto_sqs::DeleteMessageRequest {
                            queue_url: url.clone(),
                            receipt_handle,
                        }
                    );

                },
                None => {
                    let url = queue_url_from_arn(&sqs_client, arn);
                    queue_url = Some(url.clone());
                    sqs_client.delete_message(
                        &rusoto_sqs::DeleteMessageRequest {
                            queue_url: url,
                            receipt_handle,
                        }
                    );
                }
            };
        }

        Ok(())

    });
}

#[inline(always)]
pub fn handle_message<M, T>(f: impl Fn(M) -> Result<T, Error> + Clone + Send + 'static)
    where M: Message + Default
{
    lambda::logger::init();

    lambda::start(move |events: SqsEvent| {

        let (tx, rx) = channel();

        for message in events.records {
            let f = f.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let f = f.clone();
                let body = message.body.unwrap();
                let event: S3Event = serde_json::from_str(&body).unwrap();

                let messages = get_messages(event).unwrap();

                for message in messages {
                    f(message);
                }

                tx.send((message.receipt_handle.unwrap(),
                         message.event_source_arn.unwrap())).unwrap();
//                Ok(())
            });
        }

        let sqs_client =
            SqsClient::simple(rusoto_core::region::Region::UsEast1);
        let mut queue_url: Option<String> = None;

        for (receipt_handle, arn) in rx {
            match queue_url {
                Some(ref url) => {
                    sqs_client.delete_message(
                        &rusoto_sqs::DeleteMessageRequest {
                            queue_url: url.clone(),
                            receipt_handle,
                        }
                    );

                },
                None => {
                    let url = queue_url_from_arn(&sqs_client, arn);
                    queue_url = Some(url.clone());
                    sqs_client.delete_message(
                        &rusoto_sqs::DeleteMessageRequest {
                            queue_url: url,
                            receipt_handle,
                        }
                    );
                }
            };
        }

        Ok(())

    });
}

fn queue_url_from_arn(sqs: &SqsClient, arn: impl AsRef<str>) -> String {
    let queue_name = arn.as_ref().split(":").last().unwrap();

    sqs.get_queue_url(
        &GetQueueUrlRequest {
            queue_name: queue_name.into(),
            ..Default::default()
        }

    ).wait().unwrap().queue_url.unwrap()
}

#[inline(always)]
pub fn handle_proto_sqs_message<M, T>(f: impl Fn(M) -> Result<T, Error> + Clone + Send + 'static)
    where M: Message + Default
{
    lambda::logger::init();

    lambda::start(move |events: SqsEvent| {
        info!("{:#?}", events);
        let (tx, rx) = channel();

        for message in events.records.into_iter() {
            let f = f.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let body = message.body.unwrap();
                let body = base64::decode(&body).unwrap();
                let event: M = M::decode(&body).unwrap();

                f(event);

                tx.send((message.receipt_handle.unwrap(),
                         message.event_source_arn.unwrap())).unwrap();
            });
        }

        let sqs_client =
            SqsClient::simple(rusoto_core::region::Region::UsEast1);

        let mut queue_url: Option<String> = None;

        for (receipt_handle, arn) in rx {
            match queue_url {
                Some(ref url) => {
                    sqs_client.delete_message(
                        &rusoto_sqs::DeleteMessageRequest {
                            queue_url: url.clone(),
                            receipt_handle,
                        }
                    );

                },
                None => {
                    let url = queue_url_from_arn(&sqs_client, arn);
                    queue_url = Some(url.clone());
                    sqs_client.delete_message(
                        &rusoto_sqs::DeleteMessageRequest {
                            queue_url: url,
                            receipt_handle,
                        }
                    );
                }
            };
        }

        Ok(())

    });
}

pub fn events_from_s3_sns_sqs(event: SqsMessage) -> S3Event {
    let sns_event: SnsEntity = serde_json::from_str(&event.body.unwrap())
        .expect("SnsEntity");
    info!("{:#?}", sns_event);
    serde_json::from_str(sns_event.message.as_ref().unwrap())
        .expect("S3Event")
}

pub fn events_from_sns_sqs(event: SqsMessage) -> Value {
    let sns_event: SnsEntity = serde_json::from_str(&event.body.unwrap())
        .expect("SnsEntity");
    info!("{:#?}", sns_event);
    serde_json::from_str(sns_event.message.as_ref().unwrap())
        .expect("S3Event")
}


fn handle_results(thread_handles: Vec<JoinHandle<()>>,
                  rx: Receiver<Result<(String, String), Error>>) -> Result<(), Error> {
    let sqs_client =
        SqsClient::simple(rusoto_core::region::Region::UsEast1);

    let mut queue_url: Option<String> = None;
    let mut err = None;
    for result in rx {
        let (receipt_handle, arn) = match result {
            Ok((r, a)) => (r, a),
            Err(e) => {
                error!("Failed with: {:#?}", e);
                err = Some(e);
                continue
            }
        };
        match queue_url {
            Some(ref url) => {
                info!("Deleting message");
                sqs_client.delete_message(
                    &rusoto_sqs::DeleteMessageRequest {
                        queue_url: url.clone(),
                        receipt_handle,
                    }
                );
                info!("Deleted message");

            },
            None => {
                info!("Getting queue url from arn {}", arn);
                let url = queue_url_from_arn(&sqs_client, arn);
                queue_url = Some(url.clone());
                info!("Deleting message");
                sqs_client.delete_message(
                    &rusoto_sqs::DeleteMessageRequest {
                        queue_url: url,
                        receipt_handle,
                    }
                );
                info!("Deleted message");
            }
        };
    }

    if let Some(e) = err {
        error!("{:#?}", e);
        Err(e)
    } else {
        for t in thread_handles {
            t.join().expect("Thread panicked");
        }
        Ok(())
    }
}


pub fn handle_s3_sns_sqs_proto<M, T>(f: impl Fn(M) -> Result<T, Error> + Clone + Send + 'static,
                                     on_success: impl Fn(T) -> Result<(), Error> + Clone + Send + 'static)
    where M: Message + Default + 'static
{
    openssl_probe::init_ssl_cert_env_vars();

    lambda::logger::init();
    lambda::start(move |sqs_event: SqsEvent| {
        info!("handling s3 sns sqs proto");

        let (tx, rx) = channel();
        let mut thread_handles = vec![];
        info!("Handling {} sqs records", sqs_event.records.len());

        for sqs_msg in sqs_event.records {

            let f = f.clone();
            let on_success = on_success.clone();
            let tx = tx.clone();

            let sqs_receipt_handle = sqs_msg.receipt_handle.clone()
                .expect("sqs_receipt_handle");
            let sqs_arn = sqs_msg.event_source_arn.clone()
                .expect("sqs_arn");

            info!("Parsing events");
            let s3_event = events_from_s3_sns_sqs(sqs_msg);
            info!("Fetching messages");
            let messages = match get_messages(s3_event) {
                Ok(messages) => messages,
                Err(e) => {
                    error!("Error getting messages {}", e);
                    tx.send(Err(e)).unwrap();
                    continue
                }
            };

            info!("received {} messages", messages.len());

            let sqs_receipt_handle = sqs_receipt_handle.clone();
            let sqs_arn = sqs_arn.clone();
            let join_handle = std::thread::spawn(move || {

                for message in messages {

                    match f(message) {
                        Ok(subgraphs) => {
                            if let Err(e) = on_success(subgraphs) {
                                info!("Error processing message {:#?}", e);
                                tx.send(Err(e)).unwrap()
                            } else {
                                info!("Successfully processed message");
                                info!("Acking arn {}", sqs_arn);
                                tx.send(Ok((sqs_receipt_handle.clone(),
                                            sqs_arn.clone()))).unwrap();

                            }
                        }
                        Err(e) => {
                            tx.send(Err(e)).unwrap()
                        }
                    };

                }
                drop(tx);

            });

            thread_handles.push(join_handle);
        }
        drop(tx);
        handle_results(thread_handles, rx)
    });

}

pub fn handle_s3_sns_sqs_json<D, T>(f: impl Fn(D) -> Result<T, Error> + Clone + Send + 'static,
                                    on_success: impl Fn(T) -> Result<(), Error> + Clone + Send + 'static)
    where D: for<'a> Deserialize<'a>
{
    openssl_probe::init_ssl_cert_env_vars();

    lambda::logger::init();
    lambda::start(move |sqs_event: SqsEvent| {
//        info!("{:#?}", sqs_event);

        let (tx, rx) = channel();
        let mut thread_handles = vec![];
        info!("sqs records length {:#?}", sqs_event.records.len());

        for sqs_msg in sqs_event.records {

            let f = f.clone();
            let on_success = on_success.clone();
            let tx = tx.clone();

            let sqs_receipt_handle = sqs_msg.receipt_handle.clone()
                .expect("sqs_receipt_handle");
            let sqs_arn = sqs_msg.event_source_arn.clone()
                .expect("sqs_arn");
            info!("Parsing s3 events from SNS/SQS message");
            let s3_event = events_from_s3_sns_sqs(sqs_msg);
            let messages = match get_raw_messages(s3_event) {
                Ok(messages) => messages,
                Err(e) => {tx.send(Err(e)).unwrap(); continue}
            };

            info!("Received {} messages", messages.len());
            let sqs_receipt_handle = sqs_receipt_handle.clone();
            let sqs_arn = sqs_arn.clone();
            let join_handle = std::thread::spawn(move || {

                for message in messages {
                    let event: D = serde_json::from_slice(&message).unwrap();

                    match f(event) {
                        Ok(subgraphs) => {
                            if let Err(e) = on_success(subgraphs) {
                                tx.send(Err(e)).unwrap()
                            } else {
                                tx.send(Ok((sqs_receipt_handle.clone(),
                                            sqs_arn.clone()))).unwrap();

                            }
                        }
                        Err(e) => {
                            tx.send(Err(e)).unwrap()
                        }
                    };

                }
                drop(tx);

            });

            thread_handles.push(join_handle);
        }
        drop(tx);
        handle_results(thread_handles, rx)
    });

}

pub fn handle_sns_sqs_json<D, T>(f: impl Fn(D) -> Result<T, Error> + Clone + Send + 'static,
                                    on_success: impl Fn(T) -> Result<(), Error> + Clone + Send + 'static)
    where D: for<'a> Deserialize<'a>
{
    openssl_probe::init_ssl_cert_env_vars();

    lambda::logger::init();
    lambda::start(move |sqs_event: SqsEvent| {
//        info!("{:#?}", sqs_event);

        let (tx, rx) = channel();
        let mut thread_handles = vec![];
        info!("sqs records length {:#?}", sqs_event.records.len());

        for sqs_msg in sqs_event.records {

            let f = f.clone();
            let on_success = on_success.clone();
            let tx = tx.clone();

            let sqs_receipt_handle = sqs_msg.receipt_handle.clone()
                .expect("sqs_receipt_handle");
            let sqs_arn = sqs_msg.event_source_arn.clone()
                .expect("sqs_arn");
            info!("Parsing events from SNS/SQS message");
            let message = events_from_sns_sqs(sqs_msg);

            info!("{}", message);
            let sqs_receipt_handle = sqs_receipt_handle.clone();
            let sqs_arn = sqs_arn.clone();
            let join_handle = std::thread::spawn(move || {
                let event: D = serde_json::from_value(message).unwrap();

                match f(event) {
                    Ok(subgraphs) => {
                        if let Err(e) = on_success(subgraphs) {
                            tx.send(Err(e)).unwrap()
                        } else {
                            tx.send(Ok((sqs_receipt_handle.clone(),
                                        sqs_arn.clone()))).unwrap();

                        }
                    }
                    Err(e) => {
                        tx.send(Err(e)).unwrap()
                    }
                };


                drop(tx);

            });

            thread_handles.push(join_handle);
        }


        drop(tx);
        handle_results(thread_handles, rx)
    });

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        unimplemented!()
    }
}
