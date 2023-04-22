use std::rc::Rc;

use futures_lite::future::block_on;
use named_future::named_future;

#[named_future(Send)]
async fn async_drop(counter: Rc<()>) {
    drop(counter);
}

fn main() {
    block_on(async_drop(Rc::new(())));
}
