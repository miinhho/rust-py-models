use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static CHECKING: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
}

struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        CHECKING.with(|checking| {
            checking.borrow_mut().remove(self.0);
        });
    }
}

/// Evaluates hashability once for `T`, returning `false` on a recursive edge.
///
/// A recursive edge cannot prove that every reachable field is hashable, so the
/// conservative result prevents recursive model graphs from being accepted as
/// Python set elements or dictionary keys.
pub fn check<T: ?Sized>(evaluate: impl FnOnce() -> bool) -> bool {
    let id = std::any::type_name::<T>();
    if !CHECKING.with(|checking| checking.borrow_mut().insert(id)) {
        return false;
    }
    let _guard = Guard(id);
    evaluate()
}
