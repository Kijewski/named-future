use futures_lite::future::block_on;
use named_future::named_future;

#[named_future]
async fn read<'a>(value: &'a u32) -> u32 {
    *value
}

#[test]
fn ref_arg() {
    let value = 42;
    let future: Read<'_> = read(&value);
    assert_eq!(value, block_on(future));
}
