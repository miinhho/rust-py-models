use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static CHECKING: RefCell<HashSet<&'static str>> = RefCell::new(HashSet::new());
}

/// Stops recursive model graphs from recursing forever during hash checks.
pub fn check<T: ?Sized>(evaluate: impl FnOnce() -> bool) -> bool {
    let id = std::any::type_name::<T>();
    if !CHECKING.with(|checking| checking.borrow_mut().insert(id)) {
        return false;
    }
    struct Guard(&'static str);
    impl Drop for Guard {
        fn drop(&mut self) {
            CHECKING.with(|checking| {
                checking.borrow_mut().remove(self.0);
            });
        }
    }
    let _guard = Guard(id);
    evaluate()
}
