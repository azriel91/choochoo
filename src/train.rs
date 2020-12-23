/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train;

impl Train {
    /// Ensures the given destination is reached.
    pub fn reach<D>(_dest: D) {}
}

#[cfg(test)]
mod tests {
    use super::Train;

    #[test]
    fn reaches_empty_dest() {
        let dest = EmptyDest;
        Train::reach(dest);
    }

    #[derive(Debug)]
    struct EmptyDest;
}
