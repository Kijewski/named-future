use std::panic::{catch_unwind, set_hook};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures_lite::future::block_on;
use named_future::named_future;

struct IncrOnDrop(Arc<AtomicUsize>);

impl Drop for IncrOnDrop {
    fn drop(&mut self) {
        self.0.fetch_add(100, Ordering::AcqRel);
    }
}

#[named_future(Send, Sync)]
async fn increment_than_panic(incr_on_drop: IncrOnDrop) -> usize {
    incr_on_drop.0.fetch_add(10, Ordering::AcqRel);
    panic!("Oh no!");
}

#[test]
fn ensure_drop() {
    set_hook(Box::new(|_| ()));
    let counter = Arc::new(AtomicUsize::new(1));
    let future = increment_than_panic(IncrOnDrop(Arc::clone(&counter)));
    let err = catch_unwind(move || block_on(future)).unwrap_err();
    assert_ne!(*err.downcast::<&str>().unwrap(), "Oh no");
    assert_eq!(counter.load(Ordering::Acquire), 111);
}
