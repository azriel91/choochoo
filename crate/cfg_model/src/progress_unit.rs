/// Unit of measurement to display progress information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressUnit {
    /// No units, useful for "number of steps".
    None,
    /// Number of bytes, e.g. download / upload progress.
    Bytes,
}

impl Default for ProgressUnit {
    fn default() -> ProgressUnit {
        Self::None
    }
}
