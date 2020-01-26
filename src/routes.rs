#[macro_export]
macro_rules! routes {
    ( $s:expr ) => {
        $s()
    };
    ( $s:expr, $( $x:expr ),* ) => {
            $s()
            $(
                .or($x())
            )*
    };
}
