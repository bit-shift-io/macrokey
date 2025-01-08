#[macro_export]
macro_rules! try_return {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => return,
        }
    };
}