use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures_lite::future::{block_on, yield_now, zip};
use named_future::named_future;

#[named_future]
async fn count_to_three(counter: Rc<AtomicUsize>) {
    counter.as_ref().fetch_add(10, Ordering::AcqRel);
    yield_now().await;
    counter.as_ref().fetch_add(100, Ordering::AcqRel);
    yield_now().await;
    counter.as_ref().fetch_add(1000, Ordering::AcqRel);
}

#[test]
fn one_argument() {
    let counter = Rc::new(AtomicUsize::new(1));
    let future1 = count_to_three(Rc::clone(&counter));
    let future2 = count_to_three(Rc::clone(&counter));
    let future3 = count_to_three(Rc::clone(&counter));
    block_on(zip(future1, zip(future2, future3)));
    assert_eq!(counter.load(Ordering::Acquire), 3331);
}
