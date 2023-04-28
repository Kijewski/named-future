use core::{future, marker, mem, pin, ptr, task};

mod align {
    pub trait Aligner {
        type Aligned<const SIZE: usize>: core::fmt::Debug + Copy;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Int<const BYTES: usize>;

    macro_rules! impl_alignments {
        ($($ty:ident($align:literal))*) => {$(
            #[repr(C, align($align))]
            #[derive(Debug, Clone, Copy)]
            pub struct $ty<const SIZE: usize> {
                pub data: [core::mem::MaybeUninit<u8>; SIZE],
            }

            impl Aligner for Int<$align> {
                type Aligned<const SIZE: usize> = $ty<SIZE>;
            }
        )*};
    }

    impl_alignments! {
        A1(1) A2(2) A4(4) A8(8) A16(16) A32(32) A64(64) A128(128) A256(256)
        A512(512) A1024(1024) A2048(2048) A4096(4096) A8192(8192) A16384(16384)
    }
}

#[doc(hidden)]
pub unsafe trait NamedFuture {
    /// Size of the future
    const SIZE_OF: usize;

    /// Alignment of the future
    const ALIGN_OF: usize;

    /// Is the future Send?
    const SEND: bool;

    /// Is the future Sync?
    const SYNC: bool;

    /// The arguments to the generator, packed into a tuple
    type Args;

    /// Build the named future
    fn new(args: Self::Args) -> Self;
}

/// An array `[MaybeUninit<u8>; SIZE_OF]` with an alignment of (at least) `ALIGN_OF`
pub type Bytes<const SIZE_OF: usize, const ALIGN_OF: usize> =
    <align::Int<ALIGN_OF> as align::Aligner>::Aligned<SIZE_OF>;

/// [Size of](mem::size_of) the unnamed future of `Generator`
pub const fn size_of<Generator, Args, Fut>(_: &Generator) -> usize
where
    Generator: Fn(Args) -> Fut,
{
    mem::size_of::<Fut>()
}

/// [Alignment of](mem::align_of) the unnamed future of `Generator`
pub const fn align_of<Generator, Args, Fut>(_: &Generator) -> usize
where
    Generator: Fn(Args) -> Fut,
{
    mem::align_of::<Fut>()
}

/// Return `true` if `Generator` is [`Send`], otherwise undefined
pub const fn ensure_send<Generator, Args, Fut>(_: &Generator) -> bool
where
    Generator: Fn(Args) -> Fut,
    Fut: marker::Send,
{
    true
}

/// Return `true` if `Generator` is [`Sync`], otherwise undefined
pub const fn ensure_sync<Generator, Args, Fut>(_: &Generator) -> bool
where
    Generator: Fn(Args) -> Fut,
    Fut: marker::Sync,
{
    true
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
