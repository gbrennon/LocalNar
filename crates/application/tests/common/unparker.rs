#![allow(dead_code)]
use std::sync::Arc;
use std::task::Wake;
use std::thread::Thread;

/// A waker that resumes the thread blocked on a future.
pub struct Unparker(Thread);

impl Unparker {
    /// Builds a waker bound to the calling thread.
    pub fn for_current_thread() -> Arc<Self> {
        Arc::new(Self(std::thread::current()))
    }
}

impl Wake for Unparker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
