use core::{future, marker, mem, pin, ptr, task};

mod align {
    pub trait Aligner {
        type Zst: Send + Sync + Copy + Default;
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Int<const BYTES: usize>;

    macro_rules! impl_alignments {
        ($($ty:ident($align:literal))*) => {$(
            #[repr(align($align))]
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $ty;

            impl Aligner for Int<$align> {
                type Zst = $ty;
            }
        )*};
    }

    impl_alignments! {
        A1(1) A2(2) A4(4) A8(8) A16(16) A32(32) A64(64) A128(128) A256(256)
        A512(512) A1024(1024) A2048(2048) A4096(4096) A8192(8192) A16384(16384)
    }
}

#[doc(hidden)]
pub trait Layout {
    /// Size of the future
    const SIZE_OF: usize;
    /// Aligment of the future
    const ALIGN_OF: usize;
    /// Is the future Send?
    const SEND: bool;
    /// Is the future Sync?
    const SYNC: bool;
}

/// Select a type that has at lease an alignment of `BYTES`
pub type Align<const BYTES: usize> = <align::Int<BYTES> as align::Aligner>::Zst;

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

/// Transmute named future to its unnamed future
///
/// SAFETY: `Generator` must be the generator of `This`
#[inline(always)]
unsafe fn as_fut<'a, Generator, Args, Fut, This>(_: &Generator, data: &'a mut This) -> &'a mut Fut
where
    Generator: Fn(Args) -> Fut,
{
    mem::transmute(data)
}

/// Transmute pinned named future to its unnamed future
///
/// SAFETY: `Generator` must be the generator of `This`
#[inline(always)]
unsafe fn as_fut_pin<'a, Generator, Args, Fut, This>(
    _: &Generator,
    data: pin::Pin<&'a mut This>,
) -> pin::Pin<&'a mut Fut>
where
    Generator: Fn(Args) -> Fut,
{
    mem::transmute(data)
}

/// [`poll()`](future::Future::poll) for a named future
///
/// SAFETY: `Generator` must be the generator of `This`.
#[inline(always)]
pub unsafe fn poll<Generator, Args, Fut, This>(
    generator: &Generator,
    this: pin::Pin<&mut This>,
    cx: &mut task::Context<'_>,
) -> task::Poll<Fut::Output>
where
    Generator: Fn(Args) -> Fut,
    Fut: future::Future,
{
    let fut = as_fut_pin(generator, this);
    fut.poll(cx)
}

/// [`ptr::drop_in_place`] for a named future
///
/// SAFETY: `Generator` must be the generator of `This`.
#[inline(always)]
pub unsafe fn drop<Generator, Args, Fut, This>(generator: &Generator, this: &mut This)
where
    Generator: Fn(Args) -> Fut,
{
    let fut = as_fut(generator, this);
    ptr::drop_in_place(fut);
}