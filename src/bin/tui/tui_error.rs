#[derive(Debug,thiserror::Error)]
pub enum Error {
    #[error("Input string could not be parsed as the exit command")]
	Exit,
    #[error("Input string could not be parsed as the help command")]
    Help,
    #[error("Failed to parse as any command or operation: {0:?}")]
	Parse(Vec<Error>),
    #[error("Error from rpncalc library")]
    LibError(#[from] rpncalc::error::Error),
    #[error("Error reading UTF-8")]
    UTF8(#[from] std::str::Utf8Error),
    #[error("Error during IO: \n{0}")]
    IO(#[from] std::io::Error),
    #[error("Error clearing screen: \n{0}")]
    ClearScreen(#[from] clearscreen::Error),
}

pub type Result<T = (), E = Error> = std::result::Result<T,E>;