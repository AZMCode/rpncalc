use crate::error::*;
use std::str::FromStr;
use rpncalc_macros::{SimpleOp,simple_op};
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
				return Err(Error::ParseEnum(parse_errors))
			}
		}

        impl super::Command for OpEnum {
            #[allow(unused_variables)]
            fn comm(self, stack: &mut Vec<f64>, stdin: impl ::std::io::Read, stdout: impl ::std::io::Write) -> Result<Option<String>> {
                match self {
                    $(OpEnum::$v(curr_op) => {
                        curr_op.comm(stack,stdin,stdout)
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
    fn comm(self, _: &mut Vec<f64>, _: impl std::io::Read, _: impl std::io::Write) -> Result<Option<String>> {
        Ok(None)
    }
}

#[derive(Clone)]
pub struct InsNum(pub f64);

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
    
    fn comm(self, stack: &mut Vec<f64>, _: impl std::io::Read, _: impl std::io::Write) -> Result<Option<String>> {
        stack.push(self.0);
        Ok(None)
    }
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simple_op(
    name = "+ - * /",
    description = "Basic Arithmetic operations",
    input_arity = 2,
    exclude_fromstr = true
)]
pub enum Arith {
    #[simple_op(|lhs,rhs| lhs + rhs)]
    Add,
    #[simple_op(|lhs,rhs| lhs - rhs)]
    Sub,
    #[simple_op(|lhs,rhs| lhs * rhs)]
    Mul,
    #[simple_op(|lhs,rhs| lhs / rhs)]
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
#[simple_op(
    name = "Pi | E | Inf",
    description = "Constants made available for use",
    input_arity = 0
)]
pub enum Constants {
    #[simple_op(|| std::f64::consts::E)]
    E,
    #[simple_op(|| std::f64::consts::PI)]
    PI,
    #[simple_op(|| std::f64::INFINITY)]
    Inf
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simple_op(
    name = "Log10 | Log2 | LogE | Root2",
    description = "Exponential operations that take one argument",
    input_arity = 1
)]
pub enum ExponentialsUnary {
    #[simple_op(|input: f64| input.log10())]
    Log10,
    #[simple_op(|input: f64| input.log2())]
    Log2,
    #[simple_op(|input: f64| input.ln())]
    LogE,
    #[simple_op(|input: f64| input.sqrt())]
    Root2
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simple_op(
    name = "Pow | LogN | RootN",
    description = "Exponential operations that take two arguments",
    input_arity = 2
)]
pub enum ExponentialsBinary {
    #[simple_op(|lhs: f64,rhs: f64| lhs.powf(rhs))]
    Pow,
    #[simple_op(|lhs: f64,rhs: f64| rhs.log(lhs))]
    LogN,
    #[simple_op(|lhs: f64,rhs: f64| rhs.powf(lhs.recip()))]
    RootN
}

#[derive(Clone,PartialEq,SimpleOp)]
#[simple_op(
    name = "Sin | Cos | Tan | ASin | ACos | ATan",
    description = "Forward and inverse trigonometric functions",
    input_arity = 1
)]
pub enum Trigonometric {
    #[simple_op(|input: f64| input.sin())]
    Sin,
    #[simple_op(|input: f64| input.cos())]
    Cos,
    #[simple_op(|input: f64| input.tan())]
    Tan,
    #[simple_op(|input: f64| input.asin())]
    ASin,
    #[simple_op(|input: f64| input.acos())]
    ACos,
    #[simple_op(|input: f64| input.atan())]
    ATan
}

#[derive(Clone, PartialEq,SimpleOp)]
#[simple_op(
    name = "= | != | > | >= | < | <=",
    description = "Binary operators that compare two elements on the stack, the return 1 for true and 0 for false. Mostly for use with the if command.",
    input_arity = 2
)]
pub enum Cmp {
    #[simple_op(|lhs: f64,rhs: f64| if lhs == rhs { 1.0 } else { 0.0 })]
    Eq,
    #[simple_op(|lhs: f64,rhs: f64| if lhs != rhs { 1.0 } else { 0.0 })]
    Neq,
    #[simple_op(|lhs: f64,rhs: f64| if lhs >  rhs { 1.0 } else { 0.0 })]
    Gt,
    #[simple_op(|lhs: f64,rhs: f64| if lhs >= rhs { 1.0 } else { 0.0 })]
    Gte,
    #[simple_op(|lhs: f64,rhs: f64| if lhs <  rhs { 1.0 } else { 0.0 })]
    Lt,
    #[simple_op(|lhs: f64,rhs: f64| if lhs <= rhs { 1.0 } else { 0.0 })]
    Lte
}
