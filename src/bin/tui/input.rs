use crate::tui_error::*;
use rpncalc::CommandOrOp;

pub enum Input {
	Exit,
    Help,
	CommandOrOp(CommandOrOp)
}

impl std::str::FromStr for Input {
	type Err = Error;
	fn from_str(input: &str) -> Result<Input> {
		let mut errors = vec![];
		let trim_up =  input.trim().to_uppercase();
		if trim_up == "EXIT" || trim_up == "QUIT" {
			return Ok(Input::Exit);
		} else {
			errors.push(Error::Exit);
		}
        if trim_up == "HELP" || trim_up == "H" {
            return Ok(Input::Help);
        } else {
            errors.push(Error::Help);
        }
		match input.parse::<CommandOrOp>() {
			Ok(v) => return Ok(Input::CommandOrOp(v)),
			Err(e) => errors.push(e.into())
		}
		Err(Error::Parse(errors))
	}
}