/// Allows you to pull the authors for the command from your Cargo.toml at
/// compile time in the form:
/// `"author1 lastname <author1@example.com>:author2 lastname <author2@example.com>"`
///
/// You can replace the colons with a custom separator by supplying a
/// replacement string, so, for example,
/// `crate_authors!(",\n")` would become
/// `"author1 lastname <author1@example.com>,\nauthor2 lastname <author2@example.com>,\nauthor3 lastname <author3@example.com>"`
///
/// # Examples
///
/// ```no_run
/// let m = crate_authors!();
/// assert_eq!(m, "author1 lastname <author1@example.com>:author2 lastname <author2@example.com>"
/// ```
#[macro_export]
macro_rules! crate_authors {
    ($sep:expr) => {{
        static AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
        if AUTHORS.contains(':') {
            static CACHED: std::sync::OnceLock<String> = std::sync::OnceLock::new();
            let s = CACHED.get_or_init(|| AUTHORS.replace(':', $sep));
            let s: &'static str = &*s;
            s
        } else {
            AUTHORS
        }
    }};
    () => {
        env!("CARGO_PKG_AUTHORS")
    };
}

/// Allows you to pull the name from your Cargo.toml at compile time.
///
/// <div class="warning">
///
/// **NOTE:** This macro extracts the name from an environment variable `CARGO_PKG_NAME`.
/// When the crate name is set to something different from the package name,
/// use environment variables `CARGO_CRATE_NAME` or `CARGO_BIN_NAME`.
/// See [the Cargo Book](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
/// for more information.
///
/// </div>
///
/// # Examples
///
/// ```no_run
/// let m =  crate_name!();
/// assert_eq!(m, "csv_parser_rs");
/// ```
#[macro_export]
macro_rules! crate_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}
