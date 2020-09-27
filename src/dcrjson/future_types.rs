use tokio_tungstenite::tungstenite::Message;

use tokio::sync::mpsc;

use core::future::Future;

use core::task::{Context, Poll};

use core::pin::Pin;

use log::warn;

use super::jsonrpc;

pub struct AddNodeFuture {}

impl Future for AddNodeFuture {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

pub struct NotifyBlocksFuture {
    pub(super) message: mpsc::Receiver<Message>,
    pub(super) on_block_connected: Option<fn(block_header: Vec<u8>, transactions: Vec<Vec<u8>>)>,
    pub(super) on_block_disconnected: Option<fn(block_header: [u8])>,
}

impl Future for NotifyBlocksFuture {
    type Output = Result<(), String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // match self.message.poll_recv(cx) {
        //     Some(t) => {}

        //     None => {}
        // };

        match self.message.poll_recv(cx) {
            Poll::Ready(msg) => match msg {
                Some(msg) => {
                    match serde_json::from_slice(&msg.into_data()) {
                        Ok(val) => {
                            let result: jsonrpc::JsonResponse = val;

                            return Poll::Ready(Ok(()));
                        }

                        Err(e) => {
                            warn!(
                                "Error marshalling server result on notify blocks, error: {}.",
                                e
                            );

                            return Poll::Ready(Err(
                                "Error marshalling server result on notify blocks".into(),
                            ));
                        }
                    };
                }

                None => {
                    warn!("Server returned back an empty message on notify blocks.");

                    return Poll::Ready(Err("Server returned an empty result".into()));
                }
            },

            Poll::Pending => Poll::Pending,
        }

        // match self.message.try_recv() {
        //     Ok(_) => {}

        //     Err(e) => match e {
        //         mpsc::error::TryRecvError::Closed => {
        //             warn!("Notify block receiving channel closed abruptly.");

        //             return Poll::Ready(Err("slsls".into()));
        //         }

        //         mpsc::error::TryRecvError::Empty => {
        //             self.message.poll_recv(cx);
        //             return Poll::Pending;
        //         } //                return self.message.poll_recv(cx);
        //     },
        // }
    }
}
