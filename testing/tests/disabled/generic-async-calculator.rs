// "error: generic `Self` types are currently not permitted in anonymous constants"

use futures_lite::future::yield_now;
use named_future::named_future;

/// Calculate `factor1 * factor2 + summand` asynchronously
///
/// # Struct
///
/// Future returned by [`calculate`]
#[named_future(Send, Sync, Type = pub CalculateFuture, Crate = ::named_future)]
async fn calculate<T>(factor1: T, factor2: T, summand: T) -> T
where
    T: core::ops::Add + core::ops::Mul,
{
    let a = factor1;
    yield_now().await;
    let b = a * factor2;
    yield_now().await;
    let c = b + summand;
    yield_now().await;
    c
}

#[test]
fn generic_async_calculator() {
    let generator: CalculateFuture = calculate(12, 34, 56);
    let value = futures_lite::future::block_on(generator);
    assert_eq!(value, 12 * 34 + 56);
}
