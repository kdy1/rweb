#[macro_export]
macro_rules! routes {
    ( $s:expr ) => {
        $s()
    };
    ( $inject:expr; $s:expr ) => {
        $s($inject)
    };
    ( $s:expr, $( $x:expr ),* ) => {
            $s()
            $(
                .or($x())
            )*
    };
    ( $inject:expr; $s:expr, $( $x:expr ),* ) => {
            $s(inject)
            $(
                .or($x($inject))
            )*
    };
}
