//! Future types.
//! Contains all asynchronous command structures.

use {
    super::chain_command_result,
    core::future::Future,
    core::pin::Pin,
    core::task::{Context, Poll},
    log::{trace, warn},
    tokio::sync::mpsc,
};

/// Returns on-notification response from server.
pub struct NotificationsFuture {
    pub(crate) message: mpsc::Receiver<Vec<u8>>,
}

impl Future for NotificationsFuture {
    type Output = Result<(), super::RpcServerError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), super::RpcServerError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blockchain Info result.");

                    let result: chain_command_result::JsonResponse =
                        match serde_json::from_slice(&msg) {
                            Ok(val) => val,

                            Err(e) => {
                                warn!("Error marshalling Get Blockchain Info result.");
                                return Poll::Ready(Err(super::RpcServerError::Marshaller(e)));
                            }
                        };

                    if result.error.is_null() {
                        return Poll::Ready(Ok(()));
                    }

                    let error_value: String = match serde_json::from_value(result.error) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling error value.");
                            return Poll::Ready(Err(super::RpcServerError::Marshaller(e)));
                        }
                    };

                    return Poll::Ready(Err(super::RpcServerError::ServerError(error_value)));
                }

                None => return Poll::Ready(Err(super::RpcServerError::EmptyResponse)),
            },

            Poll::Pending => {
                return Poll::Pending;
            }
        };
    }
}

/// Returns GetBlockchainInfo response from server. This is an asynchronous type.
pub struct GetBlockchainInfoFuture {
    pub(crate) message: mpsc::Receiver<Vec<u8>>,
}

impl Future for GetBlockchainInfoFuture {
    type Output = Result<chain_command_result::BlockchainInfo, super::RpcServerError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<chain_command_result::BlockchainInfo, super::RpcServerError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blockchain Info result.");

                    let result: chain_command_result::JsonResponse =
                        match serde_json::from_slice(&msg) {
                            Ok(val) => val,

                            Err(e) => {
                                warn!("Error marshalling Get Blockchain Info response.");
                                return Poll::Ready(Err(super::RpcServerError::Marshaller(e)));
                            }
                        };

                    if !result.error.is_null() {
                        return Poll::Ready(Err(super::RpcServerError::ServerError(
                            result.error.to_string(),
                        )));
                    }

                    let val = match serde_json::from_value(result.result) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling Get Blockchain Info result.");
                            return Poll::Ready(Err(super::RpcServerError::Marshaller(e)));
                        }
                    };

                    return Poll::Ready(Ok(val));
                }

                None => return Poll::Ready(Err(super::RpcServerError::EmptyResponse)),
            },

            Poll::Pending => {
                return Poll::Pending;
            }
        };
    }
}
