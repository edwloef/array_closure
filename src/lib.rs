#![no_std]

use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

macro_rules! array_closure_impl {
	($array_dyn_ty:ident, $f_trait:ident, $($ref:tt)*) => {
        pub struct $array_dyn_ty<'a, const N: usize = 128, Args = (), Output = ()> {
            erased: MaybeUninit<[u8; N]>,
            call: fn($($ref)* MaybeUninit<[u8; N]>, Args) -> Output,
            drop: fn(MaybeUninit<[u8; N]>),
            _a: PhantomData<&'a (dyn $f_trait(Args) -> Output + Send + Sync + 'a)>,
        }
        impl<'a, const N: usize, Args, Output> $array_dyn_ty<'a, N, Args, Output> {
            pub fn new<F: $f_trait(Args) -> Output + Send + Sync + 'a>(f: F) -> Self {
                const { assert!(size_of::<F>() <= N) }
                let mut erased = MaybeUninit::uninit();
                unsafe { (&raw mut erased).cast::<F>().write_unaligned(f) }
                Self {
                    erased,
                    call: |erased, args| (unsafe { (&raw const erased).cast::<$($ref)* F>().read_unaligned() })(args),
                    drop: |erased| drop(unsafe { (&raw const erased).cast::<F>().read_unaligned() }),
                    _a: PhantomData,
                }
            }
            pub fn call($($ref)* self, args: Args) -> Output {
                #[allow(unused_mut)]
                let mut this = ManuallyDrop::new(self);
                (this.call)($($ref)* this.erased, args)
            }
        }
        impl<const N: usize, Args, Output> Drop for $array_dyn_ty<'_, N, Args, Output> {
            fn drop(&mut self) {
                (self.drop)(self.erased);
            }
        }
    };
}

array_closure_impl!(ArrayDynFnOnce, FnOnce,);
array_closure_impl!(ArrayDynFnMut, FnMut, &mut);
array_closure_impl!(ArrayDynFn, Fn, &);
