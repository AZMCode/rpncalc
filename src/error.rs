
#[derive(Debug,thiserror::Error)]
pub enum Error {
	#[error("Failed to parse input: {0:?}")]
	Parse(Box<Vec<Error>>),
	#[error("Failed to parse float: \n{0}")]
	ParseFloat(#[from] std::num::ParseFloatError),
    #[error("Failed to parse integer: \n{0}")]
    ParseInt(#[from] std::num::ParseIntError),
	#[error("Input string could not be parsed as the exit command")]
	Exit,
    #[error("Not enough elements in stack to run input.\n Elements in stack ({0}) < Elements needed ({1})")]
    StackEmpty(usize,usize),
    #[error("Error clearing screen: \n{0}")]
    ClearScreen(#[from] clearscreen::Error),
    #[error("Error during IO: \n{0}")]
    IO(#[from] std::io::Error),
    #[error("Could not parse token: {0}")]
    ParseToken(&'static str),
    #[error("Command tried to access the stack out of bounds, Index {0} is not within the stack sized {1}")]
    OOB(usize,usize)
}

pub type Result<T=(), E=Error> = std::result::Result<T,E>;

