use futures_lite::future::yield_now;
use named_future::named_future;

mod renamed {
    pub mod named {
        pub use named_future as future;
    }
}

/// Calculate `factor1 * factor2 + summand` asynchronously
///
/// # Struct
///
/// Future returned by [`calculate`]
#[named_future(Send, Sync, type = pub CalculateFuture, crate = renamed::named::future)]
async fn calculate(factor1: u32, factor2: u32, summand: u32) -> u32 {
    let a = factor1;
    yield_now().await;
    let b = a * factor2;
    yield_now().await;
    let c = b + summand;
    yield_now().await;
    c
}

#[test]
fn async_calculator() {
    let generator: CalculateFuture = calculate(12, 34, 56);
    let value = futures_lite::future::block_on(generator);
    assert_eq!(value, 12 * 34 + 56);
}
