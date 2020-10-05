//! Future types.
//! Contains all asynchronous command structures.

use {
    super::{chain_command_result, chain_command_result::JsonResponse},
    core::future::Future,
    core::pin::Pin,
    core::task::{Context, Poll},
    log::{trace, warn},
    tokio::sync::mpsc,
};

/// Returns on-notification response from server.
pub struct NotificationsFuture {
    pub(crate) message: mpsc::Receiver<JsonResponse>,
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

                    if msg.error.is_null() {
                        return Poll::Ready(Ok(()));
                    }

                    let error_value: String = match serde_json::from_value(msg.error) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling error value.");
                            return Poll::Ready(Err(super::RpcServerError::Marshaller(e)));
                        }
                    };

                    return Poll::Ready(Err(super::RpcServerError::ServerError(error_value)));
                }

                None => {
                    warn!("Server sent an empty response");
                    return Poll::Ready(Err(super::RpcServerError::EmptyResponse));
                }
            },

            Poll::Pending => {
                return Poll::Pending;
            }
        };
    }
}

/// Returns GetBlockchainInfo response from server. This is an asynchronous type.
pub struct GetBlockchainInfoFuture {
    pub(crate) message: mpsc::Receiver<JsonResponse>,
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

                    if !msg.error.is_null() {
                        return Poll::Ready(Err(super::RpcServerError::ServerError(
                            msg.error.to_string(),
                        )));
                    }

                    let val = match serde_json::from_value(msg.result) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling Get Blockchain Info result.");
                            return Poll::Ready(Err(super::RpcServerError::Marshaller(e)));
                        }
                    };

                    return Poll::Ready(Ok(val));
                }

                None => {
                    warn!("Server sent an empty response");
                    return Poll::Ready(Err(super::RpcServerError::EmptyResponse));
                }
            },

            Poll::Pending => {
                return Poll::Pending;
            }
        };
    }
}

pub struct GetBlockCountFuture {
    pub(crate) message: mpsc::Receiver<chain_command_result::JsonResponse>,
}

impl Future for GetBlockCountFuture {
    type Output = Result<i64, super::RpcServerError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<i64, super::RpcServerError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blocks Count result.");

                    if !msg.error.is_null() {
                        return Poll::Ready(Err(super::RpcServerError::ServerError(
                            msg.error.to_string(),
                        )));
                    }

                    let val = match serde_json::from_value(msg.result) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling Get Block Count result.");
                            return Poll::Ready(Err(super::RpcServerError::Marshaller(e)));
                        }
                    };

                    return Poll::Ready(Ok(val));
                }

                None => {
                    warn!("Server sent an empty response");
                    return Poll::Ready(Err(super::RpcServerError::EmptyResponse));
                }
            },

            Poll::Pending => {
                return Poll::Pending;
            }
        };
    }
}
