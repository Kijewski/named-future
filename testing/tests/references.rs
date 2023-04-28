use futures_lite::future::{block_on, yield_now};
use named_future::named_future;

#[named_future(Send)]
async fn sum_plus_one(mut data: [u32; 2]) -> u32 {
    let mut accu = 1;
    let accu = &mut accu;
    let [s1, s2] = &mut data;

    yield_now().await;
    *accu += *s1;
    *s1 = 0;
    yield_now().await;
    *accu += *s2;
    *s2 = 0;
    yield_now().await;

    assert_eq!(data, [0; 2]);
    *accu
}

#[test]
fn references() {
    let future = sum_plus_one([10, 100]);
    assert_eq!(block_on(future), 111);
}
