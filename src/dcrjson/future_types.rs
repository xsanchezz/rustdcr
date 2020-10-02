//! Future types.
//! Contains all asynchronous command structures.

use {
    super::chain_command_result,
    core::future::Future,
    core::pin::Pin,
    core::task::{Context, Poll},
    log::{info, trace, warn},
    tokio::sync::mpsc,
    tokio_tungstenite::tungstenite::Message,
};

/// Add node future struct.
pub struct AddNodeFuture {}

impl Future for AddNodeFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

pub struct GetBlockchainInfoFuture {
    pub(crate) message: mpsc::Receiver<Message>,
}

impl Future for GetBlockchainInfoFuture {
    type Output = Result<chain_command_result::BlockchainInfo, super::RpcJsonError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<chain_command_result::BlockchainInfo, super::RpcJsonError>> {
        match self.message.poll_recv(cx) {
            Poll::Ready(message) => match message {
                Some(msg) => {
                    trace!("Server sent a Get Blockchain Info result.");

                    let msg = msg.into_data();

                    let result: chain_command_result::JsonResponse =
                        match serde_json::from_slice(&msg) {
                            Ok(val) => val,

                            Err(e) => {
                                warn!("Error marshalling Get Blockchain Info result.");
                                return Poll::Ready(Err(super::RpcJsonError::Marshaller(e)));
                            }
                        };

                    let val = match serde_json::from_value(result.result) {
                        Ok(val) => val,

                        Err(e) => {
                            warn!("Error marshalling Get Blockchain Info result.");
                            return Poll::Ready(Err(super::RpcJsonError::Marshaller(e)));
                        }
                    };

                    return Poll::Ready(Ok(val));
                }

                None => return Poll::Ready(Err(super::RpcJsonError::EmptyResponse)),
            },

            Poll::Pending => {
                return Poll::Pending;
            }
        };
    }
}

// pub struct NotifyBlocksFuture {
//     pub(crate) message: mpsc::Receiver<Message>,
//     pub(crate) on_block_connected: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,
//     pub(crate) on_block_disconnected: Option<fn(block_header: [u8])>,
// }

// impl Future for NotifyBlocksFuture {
//     type Output = Result<(), String>;

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         // match self.message.poll_recv(cx) {
//         //     Some(t) => {}

//         //     None => {}
//         // };

//         match self.message.poll_recv(cx) {
//             Poll::Ready(msg) => match msg {
//                 Some(msg) => {
//                     match serde_json::from_slice(&msg.into_data()) {
//                         Ok(val) => {
//                             let result: rpc_types::JsonResponse = val;

//                             return Poll::Ready(Ok(()));
//                         }

//                         Err(e) => {
//                             warn!(
//                                 "Error marshalling server result on notify blocks, error: {}.",
//                                 e
//                             );

//                             return Poll::Ready(Err(
//                                 "Error marshalling server result on notify blocks".into(),
//                             ));
//                         }
//                     };
//                 }

//                 None => {
//                     warn!("Server returned back an empty message on notify blocks.");

//                     return Poll::Ready(Err("Server returned an empty result".into()));
//                 }
//             },

//             Poll::Pending => Poll::Pending,
//         }

//         // match self.message.try_recv() {
//         //     Ok(_) => {}

//         //     Err(e) => match e {
//         //         mpsc::error::TryRecvError::Closed => {
//         //             warn!("Notify block receiving channel closed abruptly.");

//         //             return Poll::Ready(Err("slsls".into()));
//         //         }

//         //         mpsc::error::TryRecvError::Empty => {
//         //             self.message.poll_recv(cx);
//         //             return Poll::Pending;
//         //         } //                return self.message.poll_recv(cx);
//         //     },
//         // }
//     }
// }
