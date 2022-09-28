#[macro_export]
macro_rules! cast {
    ($target:expr, $variant:path) => {{
        if let $variant(inner) = $target {
            inner
        } else {
            panic!("Got {:?} when casting {} to {}", $target, stringify!($target), stringify!($pat));
        }
    }};
}
