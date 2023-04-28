use futures_lite::future::block_on;
use named_future::named_future;

#[named_future]
async fn copy<'a>(dest: &'a mut u32, src: &'a u32) {
    *dest = *src;
}

#[test]
fn ref_mut_arg() {
    let mut dest = 47;
    let src = 11;
    let future: Copy<'_> = copy(&mut dest, &src);
    block_on(future);
    assert_eq!(dest, src);
}
