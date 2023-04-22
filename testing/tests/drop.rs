use std::future::ready;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures_lite::future::{block_on, or, yield_now};
use named_future::named_future;

struct IncrOnDrop(Arc<AtomicUsize>);

impl Drop for IncrOnDrop {
    fn drop(&mut self) {
        self.0.fetch_add(100, Ordering::AcqRel);
    }
}

#[named_future(Send, Sync)]
async fn increment_twice(incr_on_drop: IncrOnDrop) -> usize {
    incr_on_drop.0.fetch_add(10, Ordering::AcqRel);
    yield_now().await;
    incr_on_drop.0.fetch_add(1000, Ordering::AcqRel);
    panic!("Should not be reached!");
}

#[test]
fn ensure_drop() {
    let counter = Arc::new(AtomicUsize::new(1));
    let future = increment_twice(IncrOnDrop(Arc::clone(&counter)));
    let winner = block_on(or(future, ready(2)));
    assert_eq!(winner, 2);
    assert_eq!(counter.load(Ordering::Acquire), 111);
}
