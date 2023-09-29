use std::*;

pub type CustRes<T> = Result<T, CustomErr>;

pub struct CustomSimpleErr {}
impl<E: std::fmt::Display> From<E> for CustomSimpleErr {
    fn from(inner: E) -> Self {
        use log::*;
        error!("{}", inner);
        Self {}
    }
}

//#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomErr {
    //inner: Error
}
impl From<CustomSimpleErr> for CustomErr {
    fn from(_inner: CustomSimpleErr) -> Self {
        Self {}
    }
}

impl<E: std::fmt::Debug> From<E> for CustomErr {
    #[track_caller]
    fn from(inner: E) -> Self {
        use log::*;
        use std::backtrace::*;
        //note sometimes some line numbers are not captured and even some fn names are not captured (optimized out). The fix is to change profile debug=1
        error!(
            "{:?}\n{:?}\n{}",
            inner,
            std::panic::Location::caller(),
            Backtrace::force_capture()
        );
        Self {}
    }
}
