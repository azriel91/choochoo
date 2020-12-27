use std::{
    fmt::{self, Debug},
    future::Future,
    pin::Pin,
};

use crate::rt_model::Station;

/// Steps to run when a station is visited.
#[derive(Clone, Copy)]
pub struct VisitFn(pub fn(&'_ mut Station) -> Pin<Box<dyn Future<Output = ()> + '_>>);

impl Debug for VisitFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("VisitFn(fn(&'_ mut Station) -> Pin<Box<dyn Future<Output = ()> + '_>>)")
    }
}

impl PartialEq for VisitFn {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.0, &other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_impl_includes_all_fields() {
        let visit_fn = VisitFn(|_| Box::pin(async {}));

        assert_eq!(
            "VisitFn(fn(&'_ mut Station) -> Pin<Box<dyn Future<Output = ()> + '_>>)",
            format!("{:?}", visit_fn)
        );
    }

    #[test]
    fn partial_eq_returns_true_for_same_instance() {
        let visit_fn = VisitFn(|_| Box::pin(async {}));

        assert_eq!(&visit_fn, &visit_fn);
    }

    #[test]
    fn partial_eq_returns_false_for_different_instance() {
        let visit_fn_0 = VisitFn(|_| Box::pin(async {}));
        let visit_fn_1 = VisitFn(|_| Box::pin(async {}));

        assert_ne!(&visit_fn_0, &visit_fn_1);
    }
}
