use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Timedout,
    FetchFailed,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl std::error::Error for Error {}
