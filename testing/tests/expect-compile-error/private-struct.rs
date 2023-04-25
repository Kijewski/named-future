use futures_lite::future::block_on;

mod inner {
    #[named_future::named_future(type = Answer)]
    pub async fn answer() -> usize {
        42
    }
}

fn main() {
    let future: inner::Answer = inner::answer();
    assert_eq!(42, block_on(future));
}
