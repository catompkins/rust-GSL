//
// A rust binding for the GSL library by Guillaume Gomez (guillaume1.gomez@gmail.com)
//

#![macro_use]

#[doc(hidden)]
macro_rules! ffi_wrap {
    ($name:tt) => {
        unsafe { $crate::ffi::FFI::wrap(sys::$name as *mut _) }
    };
}

#[doc(hidden)]
macro_rules! wrap_callback {
    ($f:expr, $F:ident $(+ $lt:lifetime)?) => {{
        unsafe extern "C" fn trampoline<$($lt,)? F: Fn(f64) -> f64 $( + $lt)?>(
            x: f64,
            params: *mut ::std::os::raw::c_void,
        ) -> f64 {
            let f: &F = &*(params as *const F);
            let x = f(x);
            x
        }

        sys::gsl_function_struct {
            function: Some(trampoline::<$F>),
            params: &$f as *const _ as *mut _,
        }
    }};
}

#[doc(hidden)]
macro_rules! wrap_callback_multi {
    ($f:expr, $F:ident, $n:ident) => {{
        unsafe extern "C" fn trampoline< F: Fn(&[f64], &mut[f64])>(
            x: sys::gsl_vector,
            params: *mut ::std::os::raw::c_void,
            f: sys::gsl_vector,
        ) {
            let f: &F = &*(params as *const F);
            f(&x, &mut f)
        }

        sys::gsl_multiroot_function_struct {
            f: Some(trampoline::<$F>),
            n: $n,
            params: &$f as *const _ as *mut _,
        }
    }};
}

#[doc(hidden)]
macro_rules! wrap_callback_fdf {
    ($f:expr, $F:ident $(+ $lt:lifetime)?, $df:expr, $DF:ident $(+ $ltdf:lifetime)?, $fdf:expr, $FDF:ident) => {{
        unsafe extern "C" fn inner_f<$($lt,)? F: Fn(f64) -> f64 $( + $lt)?>(
            x: f64,
            params: *mut ::std::os::raw::c_void,
        ) -> f64 {
            let f: &F = &*(params as *const F);
            let x = f(x);
            x
        }

        unsafe extern "C" fn inner_df<$($ltdf,)? DF: Fn(f64) -> f64 $( + $ltdf)?>(
            x: f64,
            params: *mut ::std::os::raw::c_void,
        ) -> f64 {
            let df: &DF = &*(params as *const DF);
            let x = df(x);
            x
        }

        unsafe extern "C" fn inner_fdf<FDF: Fn(f64, &mut f64, &mut f64)>(
            x: f64,
            params: *mut c_void,
            y: *mut c_double,
            dy: *mut c_double,
        ) {
            let fdf: &FDF =  &*(params as *const FDF);
            fdf(x, &mut *y, &mut *dy);
        }

        sys::gsl_function_fdf {
            f: Some(inner_f::<$F>),
            df: Some(inner_df::<$DF>),
            fdf: Some(inner_fdf::<$FDF>),
            params: &($f, $df, $fdf) as *const _ as *mut _,
        }
    }};
}


#[doc(hidden)]
macro_rules! ffi_wrapper {
    ($name:ident, *mut $ty:ty, $drop:ident $(;$extra_id:ident: $extra_ty:ty => $extra_expr:expr;)* $(, $doc:expr)?) => {
        ffi_wrapper!($name, *mut $ty $(;$extra_id: $extra_ty => $extra_expr;)* $(, $doc)?);

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe { sys::$drop(self.inner) };
                self.inner = ::std::ptr::null_mut();
            }
        }
    };
    ($name:ident, *mut $ty:ty $(;$extra_id:ident: $extra_ty:ty => $extra_expr:expr;)* $(, $doc:expr)?) => {
        $(#[doc = $doc])?
        pub struct $name {
            inner: *mut $ty,
            $($extra_id: $extra_ty,)*
        }

        impl FFI<$ty> for $name {
            fn wrap(inner: *mut $ty) -> Self {
                Self { inner $(, $extra_id: $extra_expr)* }
            }

            fn soft_wrap(r: *mut $ty) -> Self {
                Self::wrap(r)
            }

            #[inline]
            fn unwrap_shared(&self) -> *const $ty {
                self.inner as *const _
            }

            #[inline]
            fn unwrap_unique(&mut self) -> *mut $ty {
                self.inner
            }
        }
    };
    ($name:ident, *const $ty:ty $(;$extra_id:ident: $extra_ty:ty => $extra_expr:expr;)* $(, $doc:expr)?) => {
        $(#[doc = $doc])?
        #[derive(Clone, Copy)]
        pub struct $name {
            inner: *const $ty,
            $($extra_id: $extra_ty,)*
        }

        impl FFI<$ty> for $name {
            fn wrap(inner: *mut $ty) -> Self {
                Self { inner $(, $extra_id: $extra_expr)* }
            }

            fn soft_wrap(inner: *mut $ty) -> Self {
                Self { inner $(, $extra_id: $extra_expr)* }
            }

            #[inline]
            fn unwrap_shared(&self) -> *const $ty {
                self.inner
            }

            fn unwrap_unique(&mut self) -> *mut $ty {
                unimplemented!()
            }
        }
    };
    () => {}
}
