// use std::time::Duration;
//
// use futures::{
//     channel::oneshot::{
//         self,
//         Receiver,
//         Sender,
//     },
//     Future,
//     FutureExt,
//     SinkExt,
//     StreamExt,
// };
//
// use tokio::net::TcpListener;
// use tokio_stream::wrappers::TcpListenerStream;
// use tonic::{
//     transport::{
//         NamedService,
//         Server,
//     },
//     Request,
//     Response,
// };
//
// use crate::{
//     execute_rpc,
//     graplinc::grapl::api::lens_subscription::v1beta1::{
//         SubscribeToLensRequest,
//         SubscribeToLensResponse,
//     },
//     protobufs::graplinc::grapl::api::lens_subscription::v1beta1::{
//         self as proto,
//         lens_subscription_service_server::{LensSubscriptionServiceServer as LensSubscriptionServiceProto},
//     },
//     protocol::{
//         healthcheck::{
//             server::init_health_service,
//             HealthcheckError,
//             HealthcheckStatus,
//         },
//         status::Status,
//     },
//     server_internals::GrpcApi,
//     SerDeError,
// };
//
// use crate::graplinc::grapl::api::lens_subscription::v1beta1::server::proto::lens_subscription_service_server::LensSubscriptionService;
//
// #[tonic::async_trait]
// pub trait LensSubscriptionApi {
//     type Error: Into<Status>;
//
//     async fn subscribe_to_lens(
//         &self,
//         request: futures::channel::mpsc::Receiver<SubscribeToLensRequest>,
//     ) -> Result<SubscribeToLensRequest, Self::Error>;
// }
//
// #[tonic::async_trait]
// impl<T> LensSubscriptionService for GrpcApi<T>
//     where
//         T: LensSubscriptionApi + Send + Sync + 'static,
// {
//     type SubscribeToLensStream = ();
//
//     async fn subscribe_to_lens(
//         &self,
//         request: Request<tonic::Streaming<SubscribeToLensRequest>>,
//     ) -> Result<Response<SubscribeToLensResponse>, tonic::Status> { // Should we be using SubscribeToLensStream instead of Response?
//         let mut proto_request = request.into_inner();
//
//         let (mut tx, rx) = futures::channel::mpsc::channel(8);
//
//         let proto_to_native_thread = async move {
//             ({
//                 while let Some(req) = proto_request.next().await {
//                     let req = req?.try_into()?;
//                     tx.send(req)
//                         .await
//                         .map_err(|e| Status::internal(e.to_string()))?;
//                 }
//                 Ok(())
//             } as Result<(), Status>)
//         };
//
//         let api_handler_thread = async move {
//             ({ self.api_server.subscribe_to_lens(rx).await.map_err(Into::into) }
//                 as Result<SubscribeToLensResponse, Status>)
//         };
//
//         let native_response: SubscribeToLensResponse =
//             match futures::try_join!(proto_to_native_thread, api_handler_thread,) {
//                 Ok((_, native_result)) => Ok(native_result),
//                 Err(err) => Err(err),
//             }?;
//
//         let proto_response = native_response.try_into().map_err(SerDeError::from)?;
//
//         Ok(Response::new(proto_response))
//     }
// }
//
// /**
//  * !!!!! IMPORTANT !!!!!
//  * This is almost entirely cargo-culted from PipelineIngressServer.
//  * Lots of opportunities to deduplicate and simplify.
//  */
// pub struct LensSubscriptionServer<T, H, F>
//     where
//         T: LensSubscriptionApi + Send + Sync + 'static,
//         H: Fn() -> F + Send + Sync + 'static,
//         F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + 'static,
// {
//     api_server: T,
//     healthcheck: H,
//     healthcheck_polling_interval: Duration,
//     tcp_listener: TcpListener,
//     shutdown_rx: Receiver<()>,
//     service_name: &'static str,
// }
//
// impl<T, H, F> LensSubscriptionServer<T, H, F>
//     where
//         T: LensSubscriptionApi + Send + Sync + 'static,
//         H: Fn() -> F + Send + Sync + 'static,
//         F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send,
// {
//     /// Construct a new gRPC server which will serve the given API
//     /// implementation on the given socket address. Server is constructed in
//     /// a non-running state. Call the serve() method to run the server. This
//     /// method also returns a channel you can use to trigger server
//     /// shutdown.
//     pub fn new(
//         api_server: T,
//         tcp_listener: TcpListener,
//         healthcheck: H,
//         healthcheck_polling_interval: Duration,
//     ) -> (Self, Sender<()>) {
//         let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
//         (
//             Self {
//                 api_server,
//                 healthcheck,
//                 healthcheck_polling_interval,
//                 tcp_listener,
//                 shutdown_rx,
//                 service_name: LensSubscriptionServiceProto::<GrpcApi<T>>::NAME,
//             },
//             shutdown_tx,
//         )
//     }
//
//     /// returns the service name associated with this service. You will need
//     /// this value to construct a HealthcheckClient with which to query this
//     /// service's healthcheck.
//     pub fn service_name(&self) -> &'static str {
//         self.service_name
//     }
//
//     /// Run the gRPC server and serve the API on this server's socket
//     /// address. Returns a ConfigurationError if the gRPC server cannot run.
//     pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
//         let (healthcheck_handle, health_service) =
//             init_health_service::<LensSubscriptionServiceProto<GrpcApi<T>>, _, _>(
//                 self.healthcheck,
//                 self.healthcheck_polling_interval,
//             )
//                 .await;
//
//         // TODO: add tower tracing, tls_config, concurrency limits
//         Ok(Server::builder()
//             .trace_fn(|request| {
//                 tracing::info_span!(
//                     "Lens Subscription",
//                     request_id = ?request.headers().get("x-request-id"),
//                     method = ?request.method(),
//                     uri = %request.uri(),
//                     extensions = ?request.extensions(),
//                 )
//             })
//             .add_service(health_service)
//             .add_service(LensSubscriptionServiceProto::new(GrpcApi::new(
//                 self.api_server,
//             )))
//             .serve_with_incoming_shutdown(
//                 TcpListenerStream::new(self.tcp_listener),
//                 self.shutdown_rx.map(|_| ()),
//             )
//             .then(|result| async move {
//                 healthcheck_handle.abort();
//                 result
//             })
//             .await?)
//     }
// }