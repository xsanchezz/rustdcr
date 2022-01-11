//! Future types.
//! Contains all asynchronous command structures.

use {
    crate::dcrjson::{
        result_types,
        result_types::{JsonResponse, RpcError},
        RpcServerError,
    },
    core::future::Future,
    core::pin::Pin,
    core::task::{Context, Poll},
    log::{trace, warn},
    tokio::sync::mpsc,
};

macro_rules! build_future {
    ($struct_name:ident, $output:ty) => {
        pub struct $struct_name {
            pub(crate) message: mpsc::Receiver<JsonResponse>,
        }

        impl $struct_name {
            pub fn new(rcvr: mpsc::Receiver<JsonResponse>) -> $struct_name {
                Self { message: rcvr }
            }
        }

        impl Future for $struct_name {
            type Output = $output;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.message.poll_recv(cx) {
                    Poll::Ready(message) => match message {
                        Some(msg) => {
                            let val = self.on_message(msg);
                            Poll::Ready(val)
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
    };
}

build_future![NotificationsFuture, Result<(), RpcServerError>];

impl NotificationsFuture {
    fn on_message(&self, message: JsonResponse) -> Result<(), RpcServerError> {
        trace!("Server sent an on_notification response");
        if message.error.is_null() {
            return Ok(());
        }

        Err(get_error_value(message.error))
    }
}

build_future![GetBlockchainInfoFuture, Result<result_types::BlockchainInfo, RpcServerError>];

impl GetBlockchainInfoFuture {
    fn on_message(
        &self,
        message: JsonResponse,
    ) -> Result<result_types::BlockchainInfo, RpcServerError> {
        trace!("server sent a Get Blockchain Info result");

        if !message.error.is_null() {
            return Err(get_error_value(message.error));
        }

        let val = match serde_json::from_value(message.result) {
            Ok(val) => val,

            Err(e) => {
                warn!("error marshalling Get Blockchain Info result");
                return Err(RpcServerError::Marshaller(e));
            }
        };

        Ok(val)
    }
}

build_future![GetBlockCountFuture, Result<i64, RpcServerError>];

impl GetBlockCountFuture {
    fn on_message(&self, message: JsonResponse) -> Result<i64, RpcServerError> {
        trace!("server sent a Get Blocks Count result");

        if !message.error.is_null() {
            return Err(get_error_value(message.error));
        }

        let val = match serde_json::from_value(message.result) {
            Ok(val) => val,

            Err(e) => {
                warn!("error marshalling Get Block Count result");
                return Err(RpcServerError::Marshaller(e));
            }
        };

        Ok(val)
    }
}

build_future![GetBlockHashFuture, Result<crate::chaincfg::chainhash::Hash, RpcServerError>];

impl GetBlockHashFuture {
    fn on_message(
        &self,
        message: JsonResponse,
    ) -> Result<crate::chaincfg::chainhash::Hash, RpcServerError> {
        trace!("server sent a Get Blocks Hash result");

        if !message.error.is_null() {
            return Err(get_error_value(message.error));
        }

        let hash: String = match serde_json::from_value(message.result) {
            Ok(val) => val,

            Err(e) => {
                warn!("error marshalling Get Block Hash result");
                return Err(RpcServerError::Marshaller(e));
            }
        };

        match crate::chaincfg::chainhash::Hash::new_from_str(&hash) {
            Ok(e) => Ok(e),

            Err(e) => {
                warn!("invalid hash bytes from server, error: {}.", e);
                Err(RpcServerError::InvalidResponse(format!("{}", e)))
            }
        }
    }
}

build_future![GetBlockVerboseFuture, Result<result_types::GetBlockVerboseResult, RpcServerError>];
impl GetBlockVerboseFuture {
    fn on_message(
        &self,
        message: JsonResponse,
    ) -> Result<result_types::GetBlockVerboseResult, RpcServerError> {
        trace!("server sent a Get Block Verbose result");
        if !message.error.is_null() {
            return Err(get_error_value(message.error));
        }

        match serde_json::from_value(message.result) {
            Ok(val) => Ok(val),

            Err(e) => {
                warn!("error marshalling Get Block Verbose result");
                Err(RpcServerError::Marshaller(e))
            }
        }
    }
}

build_future![DecodeRawTransactionFuture, Result<result_types::TxRawResult, RpcServerError>];
impl DecodeRawTransactionFuture {
    fn on_message(
        &self,
        message: JsonResponse,
    ) -> Result<result_types::TxRawResult, RpcServerError> {
        trace!("server sent a Get Block Verbose result");
        if !message.error.is_null() {
            return Err(get_error_value(message.error));
        }

        match serde_json::from_value(message.result) {
            Ok(val) => Ok(val),

            Err(e) => {
                warn!("error marshalling Get Block Verbose result");
                Err(RpcServerError::Marshaller(e))
            }
        }
    }
}

build_future![EstimateSmartFeeFuture, Result<result_types::EstimateSmartFeeResult, RpcServerError>];
impl EstimateSmartFeeFuture {
    fn on_message(
        &self,
        message: JsonResponse,
    ) -> Result<result_types::EstimateSmartFeeResult, RpcServerError> {
        trace!("server sent a Get Block Verbose result");
        if !message.error.is_null() {
            return Err(get_error_value(message.error));
        }

        match serde_json::from_value(message.result) {
            Ok(val) => Ok(val),

            Err(e) => {
                warn!("error marshalling Get Block Verbose result");
                Err(RpcServerError::Marshaller(e))
            }
        }
    }
}

fn get_error_value(error: serde_json::Value) -> RpcServerError {
    let error_value: RpcError = match serde_json::from_value(error) {
        Ok(val) => val,

        Err(e) => {
            warn!("error marshalling error value.");
            return RpcServerError::Marshaller(e);
        }
    };

    RpcServerError::ServerError(error_value)
}
