use futures_lite::future::block_on;
use named_future::named_future;

#[named_future(crate = some::path)]
async fn answer() -> usize {
    42
}

fn main() {
    let future = answer();
    assert_eq!(42, block_on(future));
}
