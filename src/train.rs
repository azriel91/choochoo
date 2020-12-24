use crate::Destination;

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub fn reach<D>(dest: D)
    where
        D: Destination,
    {
        if dest.is_reached() {
            // TODO: Report
        } else {
            // TODO: Progress
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Train;
    use crate::Destination;

    #[test]
    fn reaches_empty_dest() {
        let dest = EmptyDest;
        Train::reach(dest);
    }

    #[derive(Debug)]
    struct EmptyDest;

    impl Destination for EmptyDest {
        fn is_reached(&self) -> bool {
            true
        }
    }
}
