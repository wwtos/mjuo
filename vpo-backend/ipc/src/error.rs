#[cfg(any(windows, unix))]
use thiserror::Error;

use crate::ipc_message::IPCMessage;

#[cfg(any(windows, unix))]
#[derive(Error, Debug)]
pub enum IPCError {
    #[error("Channel send error")]
    ChannelSendError(#[from] async_std::channel::SendError<IPCMessage>),
    #[error("Channel receive error")]
    ChannelReceiveError(#[from] async_std::channel::RecvError),
    #[error("Async IO error")]
    AsyncIOError(#[from] async_std::io::Error),
}
