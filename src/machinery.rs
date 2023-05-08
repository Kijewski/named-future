use core::{future, marker, mem, pin, ptr, task};

mod align {
    pub trait Aligner {
        type Aligned<const SIZE: usize>: core::fmt::Debug + Copy;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Int<const BYTES: usize>;

    macro_rules! impl_alignments {
        ($($align:literal)*) => {$(
            const _: () = {
                #[repr(C, align($align))]
                #[derive(Debug, Clone, Copy)]
                pub struct V<const SIZE: usize> {
                    pub data: [core::mem::MaybeUninit<u8>; SIZE],
                }

                impl Aligner for Int<$align> {
                    type Aligned<const SIZE: usize> = V<SIZE>;
                }
            };
        )*};
    }

    impl_alignments!(1 2 4 8 16 32 64 128 256 512 1024 2048 4096 8192 16384 32768 65536);
}

#[doc(hidden)]
pub trait NamedFuture {
    /// Size of the future
    const SIZE_OF: usize;

    /// Alignment of the future
    const ALIGN_OF: usize;

    /// The arguments to the generator, packed into a tuple
    type Args;

    /// Build the named future
    fn new(args: Self::Args) -> Self;
}

/// An array `[MaybeUninit<u8>; SIZE_OF]` with an alignment of (at least) `ALIGN_OF`
pub type Bytes<const SIZE_OF: usize, const ALIGN_OF: usize> =
    <align::Int<ALIGN_OF> as align::Aligner>::Aligned<SIZE_OF>;

/// [Size of](mem::size_of) the unnamed future of `Generator`
#[must_use]
pub const fn size_of<Generator, Args, Fut>(_: &Generator) -> usize
where
    Generator: Fn(Args) -> Fut,
{
    mem::size_of::<Fut>()
}

/// [Alignment of](mem::align_of) the unnamed future of `Generator`
#[must_use]
pub const fn align_of<Generator, Args, Fut>(_: &Generator) -> usize
where
    Generator: Fn(Args) -> Fut,
{
    mem::align_of::<Fut>()
}

/// Implemented if `Generator` is [`Send`], otherwise undefined
pub const fn ensure_send<Generator, Args, Fut>(_: &Generator)
where
    Generator: Fn(Args) -> Fut,
    Fut: marker::Send,
{
}

/// Implemented if `Generator` is [`Sync`], otherwise undefined
pub const fn ensure_sync<Generator, Args, Fut>(_: &Generator)
where
    Generator: Fn(Args) -> Fut,
    Fut: marker::Sync,
{
}

/// [`poll()`](future::Future::poll) for a named future
///
/// SAFETY: `Generator` must be the generator of `This`.
#[inline(always)]
pub unsafe fn poll<Generator, Args, Fut, This>(
    _: &Generator,
    this: pin::Pin<&mut This>,
    cx: &mut task::Context<'_>,
) -> task::Poll<Fut::Output>
where
    Generator: Fn(Args) -> Fut,
    Fut: future::Future,
{
    let fut: pin::Pin<&mut Fut> = mem::transmute(this);
    fut.poll(cx)
}

/// [`ptr::drop_in_place`] for a named future
///
/// SAFETY: `Generator` must be the generator of `This`.
#[inline(always)]
pub unsafe fn drop<Generator, Args, Fut, This>(_: &Generator, this: &mut This)
where
    Generator: Fn(Args) -> Fut,
    Fut: future::Future,
{
    let fut: &mut Fut = mem::transmute(this);
    ptr::drop_in_place(fut);
}
