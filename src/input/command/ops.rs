use crate::error::*;
use std::str::FromStr;
use macros_organized::ops::{SimpleOp,simpleop};
use super::CommandDesc;

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
        #[allow(trivial_bounds)]
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
                        curr_op.comm(stack)
                    },)*
                }
            }
        }

        pub const OP_NAMES_DESCRIPTIONS: [(Option<&'static str>,&'static str,&'static str);${count(v)}] = [$(
            (<$v as CommandDesc>::SHORT_NAME,<$v as CommandDesc>::NAME,<$v as CommandDesc>::DESCRIPTION),
        )*];
	}
}

op_enum!{
    pub enum OpEnum {
        InsNum, Arith, Constants, ExponentialsUnary, ExponentialsBinary, Trigonometric, Cmp, NOP
    }
}

#[derive(Clone)]
pub struct NOP;

impl FromStr for NOP {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim() == "" {
            Ok(NOP)
        } else {
            Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl CommandDesc for NOP {
    const SHORT_NAME: Option<&'static str> = None;
    const NAME: &'static str = "<Empty>";
    const DESCRIPTION: &'static str = "Will do nothing";
}

impl super::Command for NOP {
    fn comm(self, _: &mut Vec<f64>) -> Result<Option<String>> {
        Ok(None)
    }
}

#[derive(Clone)]
pub struct Break;

impl CommandDesc for Break {
    const SHORT_NAME: Option<&'static str> = None;
    const NAME: &'static str = "Break";
    const DESCRIPTION: &'static str = "Unconditionally produces an error. Useful for exiting infinite loops.";
}

impl FromStr for Break {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().to_uppercase().as_str() == "BREAK" {
            Ok(Self)
        } else {
            Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl super::Command for Break {
    fn comm(self, _: &mut Vec<f64>) -> Result<Option<String>> {
        Err(Error::Break)
    }
}

#[derive(Clone)]
pub struct InsNum(f64);

impl CommandDesc for InsNum {
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

impl super::Command for InsNum {
    
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        stack.push(self.0);
        Ok(None)
    }
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simpleop(
    name = "+ - * /",
    description = "Basic Arithmetic operations",
    input_arity = 2,
    exclude_fromstr = true
)]
pub enum Arith {
    #[simpleop(|lhs,rhs| lhs + rhs)]
    Add,
    #[simpleop(|lhs,rhs| lhs - rhs)]
    Sub,
    #[simpleop(|lhs,rhs| lhs * rhs)]
    Mul,
    #[simpleop(|lhs,rhs| lhs / rhs)]
    Div
}

impl FromStr for Arith {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "+" => Ok(Arith::Add),
            "-" => Ok(Arith::Sub),
            "*" => Ok(Arith::Mul),
            "/" => Ok(Arith::Div),
            _ => Err(Error::ParseToken(Self::NAME))
        }
    }
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simpleop(
    name = "Pi | E | Inf",
    description = "Constants made available for use",
    input_arity = 0
)]
pub enum Constants {
    #[simpleop(|| std::f64::consts::E)]
    E,
    #[simpleop(|| std::f64::consts::PI)]
    PI,
    #[simpleop(|| std::f64::INFINITY)]
    Inf
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simpleop(
    name = "Log10 | Log2 | LogE | Root2",
    description = "Exponential operations that take one argument",
    input_arity = 1
)]
pub enum ExponentialsUnary {
    #[simpleop(|input: f64| input.log10())]
    Log10,
    #[simpleop(|input: f64| input.log2())]
    Log2,
    #[simpleop(|input: f64| input.ln())]
    LogE,
    #[simpleop(|input: f64| input.sqrt())]
    Root2
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simpleop(
    name = "Pow | LogN | RootN",
    description = "Exponential operations that take two arguments",
    input_arity = 2
)]
pub enum ExponentialsBinary {
    #[simpleop(|lhs: f64,rhs: f64| lhs.powf(rhs))]
    Pow,
    #[simpleop(|lhs: f64,rhs: f64| rhs.log(lhs))]
    LogN,
    #[simpleop(|lhs: f64,rhs: f64| rhs.powf(lhs.recip()))]
    RootN
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simpleop(
    name = "Sin | Cos | Tan | ASin | ACos | ATan",
    description = "Forward and inverse trigonometric functions",
    input_arity = 1
)]
pub enum Trigonometric {
    #[simpleop(|input: f64| input.sin())]
    Sin,
    #[simpleop(|input: f64| input.cos())]
    Cos,
    #[simpleop(|input: f64| input.tan())]
    Tan,
    #[simpleop(|input: f64| input.asin())]
    ASin,
    #[simpleop(|input: f64| input.acos())]
    ACos,
    #[simpleop(|input: f64| input.atan())]
    ATan
}

#[derive(Clone, PartialEq,SimpleOp)]
#[simpleop(
    name = "= | != | > | >= | < | <=",
    description = "Binary operators that compare two elements on the stack, the return 1 for true and 0 for false. Mostly for use with the if command.",
    input_arity = 2
)]
pub enum Cmp {
    #[simpleop(|lhs: f64,rhs: f64| if lhs == rhs { 1.0 } else { 0.0 })]
    Eq,
    #[simpleop(|lhs: f64,rhs: f64| if lhs != rhs { 1.0 } else { 0.0 })]
    Neq,
    #[simpleop(|lhs: f64,rhs: f64| if lhs >  rhs { 1.0 } else { 0.0 })]
    Gt,
    #[simpleop(|lhs: f64,rhs: f64| if lhs >= rhs { 1.0 } else { 0.0 })]
    Gte,
    #[simpleop(|lhs: f64,rhs: f64| if lhs <  rhs { 1.0 } else { 0.0 })]
    Lt,
    #[simpleop(|lhs: f64,rhs: f64| if lhs <= rhs { 1.0 } else { 0.0 })]
    Lte
}