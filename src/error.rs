
#[derive(Debug,thiserror::Error)]
pub enum Error {
    #[error("Failed to parse as any component of an enum: \n{0:?}")]
    ParseEnum(Vec<Error>),
	#[error("Failed to parse float: \n{0}")]
	ParseFloat(#[from] std::num::ParseFloatError),
    #[error("Failed to parse integer: \n{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Not enough elements in stack to run input.\n Elements in stack ({0}) < Elements needed ({1})")]
    StackEmpty(usize,usize),
    #[error("Could not parse token: {0}")]
    ParseToken(&'static str),
    #[error("Command tried to access the stack out of bounds, Index {0} is not within the stack sized {1}")]
    OOB(usize,usize),
    #[error("Possible infinite loop detected. Repetition reached maximum limit")]
    InfLoop,
    #[error("Unbalanced Braces")]
    UnbBraces,
    #[error("Break command was run")]
    Break,
    #[error("Error during IO: \n{0}")]
    IO(#[from] std::io::Error)
}

pub type Result<T=(), E=Error> = std::result::Result<T,E>;

