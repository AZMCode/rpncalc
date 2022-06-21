use crate::error::*;
use std::str::FromStr;

pub trait OpDesc {
    const SHORT_NAME: Option<&'static str>;
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}

pub trait Op where
	Self: std::str::FromStr + Clone,
	Self::Err: Into<Error>
{
	const INPUT_ARITY: usize;
	const OUTPUT_ARITY: usize;
	fn op(self, input: [f64;Self::INPUT_ARITY]) -> Result<[f64;Self::OUTPUT_ARITY]>;
}

macro_rules! op_enum {
	{
		pub enum OpEnum {
			$($v:ident),*
		}
	} => {
        #[derive(Clone)]
		pub enum OpEnum {
			$($v($v),)*
		}
		impl FromStr for OpEnum  where
			$(
				$v: FromStr,
				<$v as FromStr>::Err: Into<Error>,
			)*
		{
			type Err = Error;
            #[allow(unused_mut,unused_variables)]
			fn from_str(input: &str) -> Result<OpEnum> {
				let mut parse_errors: Vec<Error> = vec![];
				let trimmed_input = input.trim();
				$(
					match trimmed_input.parse::<$v>() {
						Err(e) => parse_errors.push(e.into()),
						Ok(v) => return Ok(OpEnum::$v(v))
					}
				)*
				return Err(Error::Parse(Box::new(parse_errors)))
			}
		}

        impl super::Command for OpEnum {
            #[allow(unused_variables)]
            fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
                match self {
                    $(OpEnum::$v(curr_op) => {
                        let mut input = [0.0_f64;$v::INPUT_ARITY];
                        let stack_len = stack.len();
                        if stack_len < $v::INPUT_ARITY {
                            return Err(Error::StackEmpty(stack_len, $v::INPUT_ARITY));
                        }
                        for i in 0..$v::INPUT_ARITY {
                            match stack.pop() {
                                Some(v) => input[i] = v,
                                None => panic!("Unexpectedly empty stack")
                            }
                        }
                        input.reverse();
                        let new_elms = Op::op(curr_op, input)?;
                        stack.extend(new_elms);
                        Ok(None)
                    },)*
                }
            }
        }

        pub const OP_NAMES_DESCRIPTIONS: [(Option<&'static str>,&'static str,&'static str);${count(v)}] = [$(
            (<$v as OpDesc>::SHORT_NAME,<$v as OpDesc>::NAME,<$v as OpDesc>::DESCRIPTION),
        )*];
	}
}

op_enum!{
    pub enum OpEnum {
        InsNum, Arith, Constants
    }
}

#[derive(Clone)]
pub struct InsNum(f64);

impl OpDesc for InsNum {
    const SHORT_NAME: Option<&'static str> = None;
    const NAME: &'static str = "<Number>";
    const DESCRIPTION: &'static str = "Just entering a Floating Point number will add it to the bottom of the stack";
}

impl FromStr for InsNum {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(s.trim().parse::<f64>().map(|v| InsNum(v))?)
    }
}

impl Op for InsNum {
    const INPUT_ARITY: usize = 0;
    const OUTPUT_ARITY: usize = 1;
    fn op(self, _input: [f64;0]) -> Result<[f64;1]> {
        Ok([self.0])
    }
}

#[derive(Clone)]
pub enum Arith {
    Add,
    Sub,
    Mul,
    Div
}

impl OpDesc for Arith {
    const SHORT_NAME: Option<&'static str> = None;
    const NAME: &'static str = "+ - * /";
    const DESCRIPTION: &'static str = "Basic Arithmetic operations";
}

impl FromStr for Arith {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t_s = s.trim();
        use Arith::*;
        match t_s {
            "+" => Ok(Add),
            "-" => Ok(Sub),
            "*" => Ok(Mul),
            "/" => Ok(Div),
            _ => Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl Op for Arith {
    const INPUT_ARITY: usize = 2;
    const OUTPUT_ARITY: usize = 1;
    fn op(self, [lhs,rhs]: [f64;2]) -> Result<[f64;Self::OUTPUT_ARITY]> {
        use Arith::*;
        match self {
            Add => Ok([lhs + rhs]),
            Sub => Ok([lhs - rhs]),
            Mul => Ok([lhs * rhs]),
            Div => Ok([lhs / rhs])
        }
    }
}

#[derive(Clone)]
pub struct Constants(f64);

impl OpDesc for Constants {
    const SHORT_NAME: Option<&'static str> = None;
    const NAME: &'static str = "pi / e";
    const DESCRIPTION: &'static str = "Constants made available for use";
}

impl FromStr for Constants {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let trim_up = s.trim().to_uppercase();
        match trim_up.as_str() {
            "PI" => Ok(Constants(std::f64::consts::PI)),
            "E"  => Ok(Constants(std::f64::consts::E )),
            _    => Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl Op for Constants {
    const INPUT_ARITY:  usize = 0;
    const OUTPUT_ARITY: usize = 1;
    fn op(self, input: [f64;Self::INPUT_ARITY]) -> Result<[f64;Self::OUTPUT_ARITY]> {
        InsNum(self.0).op(input)
    }
}