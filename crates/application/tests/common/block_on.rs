#![allow(dead_code)]
use std::{
    future::Future,
    task::{Context, Poll, Waker},
};

use crate::common::unparker::Unparker;

/// Drives a future to completion on the current thread.
///
/// The application layer depends on no runtime, so its tests supply this
/// minimal executor instead of pulling one in as a development dependency.
pub struct BlockOn;

impl BlockOn {
    /// Polls `future` until it resolves, parking the thread between wakeups.
    pub fn run<F: Future>(future: F) -> F::Output {
        let mut future = Box::pin(future);
        let waker = Waker::from(Unparker::for_current_thread());
        let mut context = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => std::thread::park(),
            }
        }
    }
}
