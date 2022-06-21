use crate::error::*;

pub use command::*;
mod command;

pub enum Input {
	Exit,
    NOP,
	Command(CommandEnum),
	Op(OpEnum)
}

impl std::str::FromStr for Input {
	type Err = Error;
	fn from_str(input: &str) -> Result<Input> {
		let mut errors = vec![];
		let trim_up =  input.trim().to_uppercase();
        if trim_up == "" {
            return Ok(Input::NOP);
        }
		if trim_up == "EXIT" || trim_up == "QUIT" {
			return Ok(Input::Exit);
		} else {
			errors.push(Error::Exit);
		}
		match input.parse::<CommandEnum>() {
			Ok(v) => return Ok(Input::Command(v)),
			Err(e) => errors.push(e)
		}
		match input.parse::<OpEnum>() {
			Ok(v) => return Ok(Input::Op(v)),
			Err(e) => errors.push(e)
		}
		Err(Error::Parse(Box::new(errors)))
	}
}