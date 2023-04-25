# named-future: *Give your Future a name!*

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Kijewski/named-future/ci.yml?branch=main)](https://github.com/Kijewski/named-future/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/named-future?logo=rust)](https://crates.io/crates/named-future)
![Minimum supported Rust version: 1.65](https://img.shields.io/badge/rustc-1.65+-informational?logo=rust "Minimum Supported Rust Version: 1.65")
[![License: Apache-2.0 WITH LLVM-exception](https://img.shields.io/badge/license-Apache--2.0-informational?logo=apache)](/LICENSE.md "License: Apache-2.0 WITH LLVM-exception")

Wrap a [`Future`] in a sized struct, so it can be use in traits, or as return type,
without the need for [`Box<…>`], [`dyn …`], or [`impl …`].

A simple workaround until [`#![feature(type_alias_impl_trait)]`][tait] is stabilized:

```rust,untested
/// A slow multiplication
///
/// # Struct
///
/// Future generated by [`slow_mul`]
#[named_future]
pub async fn slow_mul(factor1: u32, factor2: u32) -> u32 {
    sleep(Duration::from_secs(5)).await;
    factor1 * factor2
}
```

Expands to:

```rust,untested
/// A slow multiplication
pub fn slow_mul(factor1: u32, factor2: u32) -> SlowMul {
    ...
}

/// Future generated by [`slow_mul`]
pub struct SlowMul {
    ...
}

impl Future for SlowMul {
    type Output = u32;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        ...
    }
}
```

Additionally it will implement a `Drop`, so a dropped future will work fine,
and `Debug` for your convenience.

The proc_macro `#[named_future]` has the following optional arguments:

- **`#[named_future(Send)]`**  
  - Implement [`Send`] for the generated `struct`.
    It is currently not possible to detect automatically if the `struct` should be `Send`,
    so you have to ask for the implementation manually.
    Even so, it is ensured that the [`Future`] is send, and the compilation will fail otherwise.

- **`#[named_future(Sync)]`**  
  - Implement [`Sync`] for the generated `struct`. Please see the explanation for `Send`.

- **<code>#\[named_future(type = <em>Name</em>)\]</code>**  
  - Instead of the default name, i.e. using pascal case of the function name,
    you can override the name using this argument.
    You can also override the visibility of the `struct` using this argument: `type = pub Name`.
    By default, the visibility of the function is copied.

- **<code>#\[named_future(crate = <em>some::path</em>)\]</code>**  
  - If you have renamed the dependency in your `Cargo.toml`,
    e.g. `renamed = { package = "named-future", version = "0.0.1" }`,
    then you have to specify its name / path.
    Defaults to `::named_future`.

To add a documentation to your function, and the generated struct,
you can separate both sections with a line `/// # Struct`

The library code can be used with `#![no_std]`.

Because of limitations in rust, it is currently not possible to implement a “named future” for
generic functions: “error: generic `Self` types are currently not permitted in anonymous constants”.

Inspired by the prior work of Jun Ryung Ju: [`rename-future`]

[`Future`]: https://doc.rust-lang.org/1.65.0/core/future/trait.Future.html
[`Box<…>`]: https://doc.rust-lang.org/1.65.0/alloc/boxed/struct.Box.html
[`dyn …`]: https://doc.rust-lang.org/1.65.0/std/keyword.dyn.html
[`impl …`]: https://doc.rust-lang.org/1.65.0/std/keyword.impl.html
[`Send`]: https://doc.rust-lang.org/1.65.0/core/marker/trait.Send.html
[`Sync`]: https://doc.rust-lang.org/1.65.0/core/marker/trait.Sync.html
[tait]: https://github.com/rust-lang/rust/issues/63063
[rfc1598]: https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md
[`rename-future`]: https://github.com/ArtBlnd/rename-future/tree/20c9d44726fd9f148f118cc260b713ce3d609ba2
