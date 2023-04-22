use futures_lite::future::yield_now;
use named_future::named_future;

#[named_future(Send, Sync)]
async fn yield_twice() {
    yield_now().await;
    yield_now().await;
}

#[test]
fn no_arguments() {
    futures_lite::future::block_on(yield_twice());
}
