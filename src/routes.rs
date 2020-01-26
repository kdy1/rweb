/// Helper macro to chain multiple routes with .or(route()) between them.
///
/// # Example - single use with data injection
///
/// ```ignore
/// struct DbConnection;
/// fn display_user(#[data] user: DbConnection) {}
/// let db_connection = DbConnection;
/// assert_eq!(routes![db_connection; display_user], display_user(db_connection));
/// ```
///
/// # Example - Multiple routes
///
/// ```ignore
/// fn info() {}
/// fn ping() {}
/// assert_eq!(routes![ping, info], ping().or(info()));
/// ```
///
/// # Example - Multiple routes with data injection
///
/// ```ignore
/// struct DbConnection;
/// fn display_user(#[data] db: DbConnection) {}
/// fn display_users(#[data] db: DbConnection) {}
/// let db_connection = DbConnection;
/// assert_eq!(routes![db_connection; display_user, display_users], display_user(db_connection).or(display_users(db_connection)));
/// ```
///
/// # Example - Multiple routes chaining with data injection
///
/// ```ignore
/// struct DbConnection;
/// fn ping() {}
/// fn info() {}
/// fn display_user(#[data] db: DbConnection) {}
/// let db_connection = DbConnection;
/// assert_eq!(routes![ping, info].or(routes![db_connection; display_user]), ping().or(info()).or(display_user(db_connection)));
/// ```
#[macro_export]
macro_rules! routes {
    ( $s:expr ) => {
        /// This is used when you use routes! with a single route without any data; I.e routes!(ping)
        $s()
    };
    ( $inject:expr; $s:expr ) => {
        /// This is used when you use routes! with a single route and want to pass some data to it; I.e routes!(db_connection; get_user)
        $s($inject)
    };
    ( $s:expr, $( $x:expr ),* ) => {
        /// This is used when you use routes! with multiple routes without any data: I.e routes!(ping, get_users, get_users)
            $s()
            $(
                .or($x())
            )*
    };
    ( $inject:expr; $s:expr, $( $x:expr ),* ) => {
        /// This is used when you use routes! with multiple routes and want to pass some data to it: I.e routes!(db_connection; ping, get_users, get_users)
            $s(inject)
            $(
                .or($x($inject))
            )*
    };
}
