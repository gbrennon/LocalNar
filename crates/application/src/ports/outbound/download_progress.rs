use domain::ByteLength;

/// How far a transfer has got.
///
/// The value is a snapshot, not an entry in a log: a consumer that only ever
/// keeps the most recent one still renders correctly, which is what lets a
/// user interface sample at its own frame rate instead of draining a queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadProgress {
    /// The transfer began and the total size is known.
    Started { total: ByteLength },
    /// Bytes have landed since the transfer began.
    Advanced {
        transferred: ByteLength,
        total: ByteLength,
    },
    /// The transfer finished; no further reports follow.
    Finished,
}
