//! Future types.
//! Contains all asynchronous command structures.

use {
    crate::dcrjson::{
        types,
        types::{JsonResponse, RpcError},
        RpcServerError,
    },
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
    type Output = Result<(), RpcServerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), RpcServerError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blockchain Info result.");

                    if msg.error.is_null() {
                        return Poll::Ready(Ok(()));
                    }

                    Poll::Ready(Err(get_error_value(msg.error)))
                }

                None => {
                    warn!("Server sent an empty response");
                    Poll::Ready(Err(RpcServerError::EmptyResponse))
                }
            },

            Poll::Pending => Poll::Pending,
        }
    }
}

/// Returns GetBlockchainInfo response from server. This is an asynchronous type.
pub struct GetBlockchainInfoFuture {
    pub(crate) message: mpsc::Receiver<JsonResponse>,
}

impl Future for GetBlockchainInfoFuture {
    type Output = Result<types::BlockchainInfo, RpcServerError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<types::BlockchainInfo, RpcServerError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blockchain Info result.");

                    if !msg.error.is_null() {
                        return Poll::Ready(Err(get_error_value(msg.error)));
                    }

                    let val = match serde_json::from_value(msg.result) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling Get Blockchain Info result.");
                            return Poll::Ready(Err(RpcServerError::Marshaller(e)));
                        }
                    };

                    Poll::Ready(Ok(val))
                }

                None => {
                    warn!("Server sent an empty response");
                    Poll::Ready(Err(RpcServerError::EmptyResponse))
                }
            },

            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct GetBlockCountFuture {
    pub(crate) message: mpsc::Receiver<JsonResponse>,
}

impl Future for GetBlockCountFuture {
    type Output = Result<i64, RpcServerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<i64, RpcServerError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blocks Count result.");

                    if !msg.error.is_null() {
                        return Poll::Ready(Err(get_error_value(msg.error)));
                    }

                    let val = match serde_json::from_value(msg.result) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling Get Block Count result.");
                            return Poll::Ready(Err(RpcServerError::Marshaller(e)));
                        }
                    };

                    Poll::Ready(Ok(val))
                }

                None => {
                    warn!("Server sent an empty response");
                    Poll::Ready(Err(RpcServerError::EmptyResponse))
                }
            },

            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct GetBlockHashFuture {
    pub(crate) message: mpsc::Receiver<JsonResponse>,
}

impl Future for GetBlockHashFuture {
    type Output = Result<crate::chaincfg::chainhash::Hash, RpcServerError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<crate::chaincfg::chainhash::Hash, RpcServerError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blocks Count result.");

                    if !msg.error.is_null() {
                        return Poll::Ready(Err(get_error_value(msg.error)));
                    }

                    let hash: String = match serde_json::from_value(msg.result) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling Get Block Count result.");
                            return Poll::Ready(Err(RpcServerError::Marshaller(e)));
                        }
                    };

                    match crate::chaincfg::chainhash::Hash::new_from_str(&hash) {
                        Ok(e) => Poll::Ready(Ok(e)),

                        Err(e) => {
                            warn!("Invalid hash bytes from server, error: {}.", e);
                            Poll::Ready(Err(RpcServerError::InvalidResponse(format!("{}", e))))
                        }
                    }
                }

                None => {
                    warn!("Server sent an empty response");
                    Poll::Ready(Err(RpcServerError::EmptyResponse))
                }
            },

            Poll::Pending => Poll::Pending,
        }
    }
}

fn get_error_value(error: serde_json::Value) -> RpcServerError {
    let error_value: RpcError = match serde_json::from_value(error) {
        Ok(val) => val,

        Err(e) => {
            warn!("Error marshalling error value.");
            return RpcServerError::Marshaller(e);
        }
    };

    RpcServerError::ServerError(error_value)
}
