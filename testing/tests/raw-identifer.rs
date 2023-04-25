#![allow(non_camel_case_types)]

use futures_lite::future::block_on;
use named_future::named_future;

#[named_future(type = r#struct)]
async fn r#async() -> usize {
    42
}

#[test]
fn raw_identifer() {
    let future: r#struct = r#async();
    assert_eq!(42, block_on(future));
}
