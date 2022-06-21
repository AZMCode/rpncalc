use crate::error::*;
use std::str::FromStr;

mod ops;
pub use ops::*;

pub trait Command where
    Self: std::str::FromStr + Clone,
    Self::Err: Into<Error>
{
	fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>>;
}

impl<T> Command for T where
    T: ops::Op,
    T::Err: Into<Error>,
    [f64;T::INPUT_ARITY]: Sized,
    [f64;T::OUTPUT_ARITY]: Sized
{
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        let mut input = [0.0_f64;Self::INPUT_ARITY];
        let stack_len = stack.len();
        if stack_len < Self::INPUT_ARITY {
            return Err(Error::StackEmpty(stack_len, Self::INPUT_ARITY));
        }
        for i in 0..Self::INPUT_ARITY {
            match stack.pop() {
                Some(v) => input[i] = v,
                None => panic!("Unexpectedly empty stack")
            }
        }
        input.reverse();
        let new_elms = ops::Op::op(self, input)?;
        stack.extend(new_elms);
        Ok(None)
    }
}

macro_rules! command_enum {
    {
        pub enum CommandEnum {
            $($v: ident),*
        }
    } => {
        #[derive(Clone)]
        pub enum CommandEnum {
            $($v($v),)*
        }

        impl Command for CommandEnum {
            #[allow(unused_variables)]
            fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
                match self {
                    $(CommandEnum::$v(v) => Command::comm(v,stack),)*
                }
            }
        }

        impl FromStr for CommandEnum {
            type Err = Error;
            #[allow(unused_variables,unused_mut)]
            fn from_str(input: &str) -> Result<CommandEnum> {
				let mut parse_errors: Vec<Error> = vec![];
				let trimmed_input = input.trim();
				$(
					match trimmed_input.parse::<$v>() {
						Err(e) => parse_errors.push(e.into()),
						Ok(v) => return Ok(CommandEnum::$v(v))
					}
				)*
				return Err(Error::Parse(Box::new(parse_errors)))
			}
        }

        pub const COMM_NAMES_DESCRIPTIONS: [(Option<&'static str>,&'static str,&'static str);${count(v)}] = [$(
            (<$v as OpDesc>::SHORT_NAME,<$v as OpDesc>::NAME,<$v as OpDesc>::DESCRIPTION),
        )*];
    }
}

command_enum!{
    pub enum CommandEnum {
        Help, Drop, Dup, Swap, Repeat
    }
}

#[derive(Clone)]
pub struct Help;

impl OpDesc for Help {
    const SHORT_NAME: Option<&'static str> = Some("H");
    const NAME: &'static str = "Help";
    const DESCRIPTION: &'static str = "Shows the current help page";
}

impl FromStr for Help {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trim_up_input = input.trim().to_uppercase();
        if trim_up_input == "H" || trim_up_input == "HELP" {
            Ok(Help)
        } else {
            Err(Error::ParseToken(Help::NAME))
        }
    }
}

impl Command for Help {
    fn comm(self, _stack: &mut Vec<f64>) -> Result<Option<String>> {
        let [commands,ops] = [COMM_NAMES_DESCRIPTIONS.as_slice(),ops::OP_NAMES_DESCRIPTIONS.as_slice()].map(|t|
            t.iter().fold(String::new(),|acc,(short_name_op,name,desc)|
                match short_name_op {
                    Some(short_name) => format!("{acc} {short_name} / {name} : {desc}\n" ),
                    None => format!("{acc} {name} : {desc}\n")
                }
            )
        );
        Ok(Some(format!(include_str!("help_format_str.txt"),commands,ops)))
    }
}

#[derive(Clone)]
pub enum Drop {
    Some(usize),
    All
}

impl OpDesc for Drop {
    const SHORT_NAME: Option<&'static str> = Some("D [int]");
    const NAME: &'static str = "Drop [int]";
    const DESCRIPTION: &'static str = "Takes a number of items to drop from the bottom of the stack. If the argument is \"all\", all values will be dropped. Otherwise drops one.";
}

impl FromStr for Drop {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().split_once(' ') {
            Some((drop_key,drop_arg)) => {
                let drop_key_trim_up = drop_key.trim().to_uppercase();
                let drop_arg_trim_up = drop_arg.trim().to_uppercase();
                if drop_key_trim_up == "DROP" || drop_key_trim_up == "D" {
                    if drop_arg_trim_up == "ALL" {
                        Ok(Drop::All)
                    } else {
                        let amount = drop_arg_trim_up.parse::<usize>()?;
                        Ok(Drop::Some(amount))
                    }
                } else {
                    Err(Error::ParseToken(Self::NAME))
                }
            },
            None => {
                let s_trim_up = s.trim().to_uppercase();
                if s_trim_up == "DROP" || s_trim_up == "D" {
                    Ok(Drop::Some(1))
                } else {
                    Err(Error::ParseToken(Self::NAME))
                }
            }
        }
    }
}

impl Command for Drop {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        match self {
            Drop::Some(amount) => {
                let stack_len = stack.len();
                    if amount > stack_len {
                        stack.clear();
                    } else {
                        stack.truncate(stack_len - amount);
                    }
            },
            Drop::All => stack.clear()
        }
        Ok(None)
    }
}

#[derive(Clone)]
pub struct Dup(usize);

impl OpDesc for Dup {
    const SHORT_NAME: Option<&'static str> = Some("Dup [int]");
    const NAME: &'static str = "Duplicate [int]";
    const DESCRIPTION: &'static str = "Duplicates the last element in the stack a specified amount of times, or by default one.";
}

impl FromStr for Dup {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.trim().split_once(' ') {
            Some((dup_key,dup_arg)) => {
                let dup_key_trim_up = dup_key.trim().to_uppercase();
                let dup_arg_trim = dup_arg.trim();
                if dup_key_trim_up == "DUP" || dup_key_trim_up == "DUPLICATE" {
                    let amount = dup_arg_trim.parse::<usize>()?;
                    Ok(Dup(amount))
                } else {
                    Err(Error::ParseToken(Self::NAME))
                }
            },
            None => {
                let s_trim_up = s.trim().to_uppercase();
                if s_trim_up == "DUP" || s_trim_up == "DUPLICATE" {
                    Ok(Dup(1))
                } else {
                    Err(Error::ParseToken(Self::NAME))
                }
            }
        }
    }
}

impl Command for Dup {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        let Dup(amount) = self;
        if let Some(&last_elm) = stack.last() {
            for _ in 0..amount {
                stack.push(last_elm);
            }
        }
        Ok(None)
    }
}

#[derive(Clone)]
pub enum Swap {
    Specified(usize,usize),
    LastTwo
}

impl OpDesc for Swap {
    const SHORT_NAME: Option<&'static str> = Some("S");
    const NAME: &'static str = "Swap";
    const DESCRIPTION: &'static str = "Swaps the position of any two values in the stack. If argument is not provided, it swaps the last element with the previous one";
}

impl FromStr for Swap {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let trim_up_s = s.trim().to_uppercase();
        match trim_up_s.split_once(' ').map(|(key,rest)| {
            let (from,to) = rest.trim().split_once(' ')?;
            Some((key.trim(),from.trim(),to.trim()))
        }) {
            Some(Some((key,from,to))) => {
                let key_upper = key.to_uppercase();
                if key_upper == "SWAP" || key_upper == "S" {
                    let from_int = from.parse::<usize>()?;
                    let to_int = to.parse::<usize>()?;
                    Ok(Swap::Specified(from_int,to_int))
                } else {
                    Err(Error::ParseToken(Self::NAME))
                }
            },
            Some(None) | None => {
                if trim_up_s == "SWAP" || trim_up_s == "S" {
                    Ok(Swap::LastTwo)
                } else {
                    Err(Error::ParseToken(Self::NAME))
                }
            }
        }
    }
}

impl Command for Swap {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        match self {
            Swap::Specified(from, to) => {
                let stack_len = stack.len();
                if stack_len > from {
                    if stack_len > to {
                        stack.swap(stack_len - from - 1, stack_len - to - 1);
                        Ok(None)
                    } else {
                        Err(Error::OOB(to,stack_len))
                    }
                } else {
                    Err(Error::OOB(from,stack_len))
                }
            },
            Swap::LastTwo => {
                let stack_len = stack.len();
                if stack_len >= 2 {
                    stack.swap(stack_len - 1, stack_len - 2);
                    Ok(None)
                } else {
                    Err(Error::StackEmpty(stack_len, 2))
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum CommandOrOp {
    Command(CommandEnum),
    Op(OpEnum)
}

impl FromStr for CommandOrOp {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut errors = vec![];
        match s.parse::<CommandEnum>() {
            Ok(v) => return Ok(CommandOrOp::Command(v)),
            Err(e) => {
                errors.push(e);
                match s.parse::<OpEnum>() {
                    Ok(v) => return Ok(CommandOrOp::Op(v)),
                    Err(e) => errors.push(e)
                }
            }
        }
        Err(Error::Parse(Box::new(errors)))
    }
}

impl Command for CommandOrOp {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        match self {
            CommandOrOp::Command(c) => c.comm(stack),
            CommandOrOp::Op(o) => o.comm(stack)
        }
    }
}

#[derive(Clone)]
pub enum Repeat {
    Bounded(usize,Box<CommandOrOp>),
    Unbounded(Box<CommandOrOp>)
}

impl OpDesc for Repeat {
    const SHORT_NAME: Option<&'static str> = Some("R(int) | R");
    const NAME: &'static str = "Repeat(int) | Repeat";
    const DESCRIPTION: &'static str = "Repeats a command a specified number of times, or if argument not provided, until an error is yielded";
}

impl FromStr for Repeat {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[derive(Clone)]
        enum RepeatParseState {
            Start,
            Key(bool),
            KeyParenOpen(String),
            ParenStop(usize,String),
            NoParenStop(String),
            KeyError
        }
        use RepeatParseState::*;
        let mut parsing_state = Start;
        let mut s_chars = s.chars();
        while let Some(c) = s_chars.next() {
            parsing_state = match parsing_state.clone() {
                Start => if c.to_string().to_uppercase() == "R" {
                    Key(true)
                } else {
                    KeyError
                },
                Key(is_known_long) => match c.to_string().to_uppercase().as_str() {
                    "E" => if is_known_long {
                            KeyError
                    } else {
                        let mut successful = true;
                        for expected_c in "PEAT".chars() {
                            match s_chars.next() {
                                Some(actual_c) => if expected_c != actual_c {
                                    successful = false;
                                    break;
                                },
                                None => { successful = false; break; }
                            }
                        }
                        if successful {
                            Key(true)
                        } else {
                            KeyError
                        }
                    },
                    "(" => KeyParenOpen(String::new()),
                    " " => {
                        let mut out = String::new();
                        while let Some(c) = s_chars.next() {
                            out.push(c);
                        }
                        NoParenStop(out)
                    },
                    _ => KeyError
                },
                KeyParenOpen(mut partial_str) => if c.is_alphanumeric() {
                    partial_str.push(c);
                    KeyParenOpen(partial_str)
                } else if c == ')' {
                    let amount = partial_str.parse::<usize>()?;
                    if !(matches!(s_chars.next(),Some(' '))) {
                        KeyError
                    } else {
                        let mut rest_str = String::new();
                        while let Some(inner_c) = s_chars.next() {
                            rest_str.push(inner_c);
                        }
                        ParenStop(amount,rest_str)
                    }
                } else {
                    KeyError
                },
                ParenStop(_,_) | NoParenStop(_) => panic!("State machine did not consume iterator before stopping"),
                KeyError  => break
            }
        }
        match parsing_state {
            KeyError => Err(Error::ParseToken(Self::NAME)),
            ParenStop(amount,rest) => Ok(Repeat::Bounded(amount,Box::new(rest.parse::<CommandOrOp>()?))),
            NoParenStop(rest) => Ok(Repeat::Unbounded(Box::new(rest.parse::<CommandOrOp>()?))),
            _ => unreachable!()
        }
    }
}

impl Command for Repeat {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        match self {
            Repeat::Unbounded(c) => Ok(Some(loop {
                match c.clone().comm(stack) {
                    Ok(_) => (),
                    Err(e) => break format!("Ended Repetitions with the following error: \n{e}")
                }
            })),
            Repeat::Bounded(reps, c) => {
                let mut output = Ok(None);
                for _ in 0..reps {
                    output = c.clone().comm(stack);
                    if output.is_err() { break; }
                }
                output
            }
        }
    }
}