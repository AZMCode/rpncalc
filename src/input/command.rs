use crate::error::*;
use std::str::FromStr;

mod ops;
pub use ops::*;

const MAX_REPETITIONS: usize = 1_000_000;

pub trait CommandDesc {
    const SHORT_NAME: Option<&'static str>;
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}

pub trait Command where
    Self: std::str::FromStr + Clone,
    Self::Err: Into<Error>
{
	fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>>;
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
            (<$v as CommandDesc>::SHORT_NAME,<$v as CommandDesc>::NAME,<$v as CommandDesc>::DESCRIPTION),
        )*];
    }
}

command_enum!{
    pub enum CommandEnum {
        Help, Drop, Dup, Swap, Reverse, Repeat, Chain, Conditional, Break, Input, Display, Print
    }
}

#[derive(Clone)]
pub struct Help;

impl CommandDesc for Help {
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
                    Some(short_name) => format!("{acc} {short_name} | {name} : {desc}\n" ),
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

impl CommandDesc for Drop {
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
                if stack_len == 0 && amount != 0 {
                    Err(Error::StackEmpty(stack_len, amount))
                } else if amount >= stack_len {
                    stack.clear();
                    Ok(None)
                } else {
                    stack.truncate(stack_len - amount);
                    Ok(None)
                }
            },
            Drop::All => {
                stack.clear();
                Ok(None)
            }
        }
    }
}

#[derive(Clone)]
pub struct Dup(usize);

impl CommandDesc for Dup {
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
            Ok(None)
        } else {
            Err(Error::StackEmpty(0, 1))
        }
    }
}

#[derive(Clone)]
pub enum Swap {
    Specified(usize,usize),
    LastTwo
}

impl CommandDesc for Swap {
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
pub struct Reverse;

impl CommandDesc for Reverse {
    const SHORT_NAME: Option<&'static str> = Some("Rev");
    const NAME: &'static str = "Reverse";
    const DESCRIPTION: &'static str = "Reverses the order of the stack";
}

impl FromStr for Reverse {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_trim_up = s.trim().to_uppercase();
        match s_trim_up.as_str() {
            "REV" | "REVERSE" => Ok(Reverse),
            _ => Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl Command for Reverse {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        stack.reverse();
        Ok(None)
    }
}

/* Disabled until a rustc bug is fixed: */

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

impl CommandDesc for Repeat {
    const SHORT_NAME: Option<&'static str> = Some("R(int) | R");
    const NAME: &'static str = "Repeat(int) | Repeat";
    const DESCRIPTION: &'static str = "Repeats a command a specified number of times, or if argument not provided, until an error is yielded or a precompiled limit is reached";
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
        let s_trim_up = s.trim().to_uppercase();
        let mut s_chars = s_trim_up.chars();
        while let Some(c) = s_chars.next() {
            parsing_state = match parsing_state.clone() {
                Start => if c.to_string().to_uppercase() == "R" {
                    Key(false)
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
                KeyError  =>  break
            }
        }
        match parsing_state {
            KeyError | Start => Err(Error::ParseToken(Self::NAME)),
            ParenStop(amount,rest) => Ok(Repeat::Bounded(amount,Box::new(rest.parse()?))),
            NoParenStop(rest) => Ok(Repeat::Unbounded(Box::new(rest.parse()?))),
            _ => unreachable!()
        }
    }
}

impl Command for Repeat {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        match self {
            Repeat::Unbounded(c) => {
                let mut rep_count = 0;
                loop {
                    if rep_count >= MAX_REPETITIONS {
                        break Err(Error::InfLoop)
                    }
                    match c.clone().comm(stack) {
                        Ok(_) => (),
                        Err(e) => break Ok(Some(format!("Ended Repetitions with the following error: \n{e}")))
                    }
                    rep_count += 1;
                }
            },
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

#[derive(Clone)]
pub struct Chain(Vec<CommandOrOp>);

impl Chain {
    pub fn from_bare(input: &str) -> Result<Self> {
        let mut inner_pieces = vec![];
        let mut brackets: usize = 0;
        let mut curr_piece = String::new();
        macro_rules! bail { () => {{ return Err(Error::UnbBraces) }} }
        for c in input.trim().chars() {
            match c {
                '[' => { brackets += 1; curr_piece.push('['); },
                ']' => if brackets == 0 {
                    bail!()
                } else {
                    brackets -= 1;
                    curr_piece.push(']')
                },
                ';' if brackets == 0 => inner_pieces.push(std::mem::take(&mut curr_piece)),
                c => curr_piece.push(c)
            }
        }
        inner_pieces.push(curr_piece);
        if brackets != 0 {
            bail!()
        }
        Ok(Chain(inner_pieces.into_iter().map(|inner_piece| inner_piece.parse::<CommandOrOp>()).collect::<Result<Vec<_>,_>>()?))
    }
}

impl CommandDesc for Chain {
    const SHORT_NAME: Option<&'static str> = None;
    const NAME: &'static str = "[ <Command Or Op>; ... ]";
    const DESCRIPTION: &'static str = "Allows you to chain commands or operations together";
}

impl FromStr for Chain {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_trim = s.trim();
        if s.starts_with('[') && s.ends_with(']') {
            let inner_text = &s_trim[1..(s.len() - 1)];
            Self::from_bare(inner_text)
        } else {
            Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl Command for Chain {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        let mut out = None;
        for c in self.0 {
            out = c.comm(stack)?;
        }
        Ok(out)
    }
}

#[derive(Clone)]
enum ConditionalKind {
    If,
    Try
}

#[derive(Clone)]
pub struct Conditional(ConditionalKind,[Chain;2]);

impl CommandDesc for Conditional {
    const SHORT_NAME: Option<&'static str> = None;
    const NAME: &'static str = "if [ <First Command or Op> ] [ <Second Command or Op> ] | try [ <First Command or Op> ] [ <Command or Op on failure> ]";
    const DESCRIPTION: &'static str = "If: Pops the value at the top of the stack, executes the first command if nonzero, otherwise the other. Try: Runs the first command. If an error occurs during execution, the command is interrupted and the second command is run.";
}

impl FromStr for Conditional {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trim_up_s = s.trim().to_uppercase();
        let mut trim_up_s_chars = trim_up_s.chars();
        #[derive(Clone)]
        enum ParseState {
            Start,
            First(ConditionalKind,String),
            Second(ConditionalKind,[String;2]),
            End(ConditionalKind,[String;2])
        }
        use ParseState::*;
        let mut brackets: usize = 1;
        let mut state = Start;
        macro_rules! bail { () => {{ return Err(Error::ParseToken(Self::NAME)) }} }
        while let Some(c)  = trim_up_s_chars.next() {
            state = match state.clone() {
                Start => {
                    if let Some(second_char) = trim_up_s_chars.next() {
                        let cond_kind = if c == 'I' && second_char == 'F' {
                            ConditionalKind::If
                        } else if c == 'T' && second_char == 'R' && Some('Y') == trim_up_s_chars.next() {
                            ConditionalKind::Try
                        } else { bail!() };
                        loop {
                            if let Some(poss_space_c) = trim_up_s_chars.next() {
                                if !poss_space_c.is_ascii_whitespace() {
                                    if poss_space_c == '[' {
                                        break First(cond_kind,poss_space_c.to_string());
                                    } else { bail!() }
                                }
                            } else { bail!() }
                        }
                    } else { bail!() }
                },
                First(cond_kind,mut s) => {
                    let mut inner_c = c;
                    loop {
                        match inner_c {
                            '[' => { brackets += 1; s.push('[') },
                            ']' => if brackets == 1 {
                                s.push(']');
                                break loop {
                                    if let Some(in_bet_whitespace) = trim_up_s_chars.next() {
                                        if in_bet_whitespace == '[' {
                                            break Second(cond_kind,[s,'['.to_string()]);
                                        } else if !in_bet_whitespace.is_ascii_whitespace() { bail!() }
                                    } else { bail!() }
                                };
                            } else { brackets -= 1; s.push(']'); },
                            c => s.push(c)
                        };
                        if let Some(new_inner_c) = trim_up_s_chars.next() {
                            inner_c = new_inner_c;
                        } else {
                            bail!()
                        }
                    }
                },
                Second(cond_kind,[s_first,mut s]) => {
                    let mut inner_c = c;
                    loop {
                        match inner_c {
                            '[' => { brackets += 1; s.push('[') },
                            ']' => if brackets == 1 {
                                s.push(']');
                                break End(cond_kind,[s_first,s]);
                            } else { brackets -= 1; s.push(']') },
                            c => s.push(c)
                        }
                        if let Some(new_inner_c) = trim_up_s_chars.next() {
                            inner_c = new_inner_c;
                        } else {
                            bail!()
                        }
                    }
                },
                End(_,_) => bail!()
            }
        }
        match state {
            Start => bail!(),
            End(cond_kind,[first,second]) => Ok(Conditional(cond_kind,[first.parse::<Chain>()?,second.parse::<Chain>()?])),
            _ => unreachable!()
        }
    }
}

impl Command for Conditional {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        let [first_chain,second_chain] = self.1;
        match self.0 {
            ConditionalKind::If => if let Some(v) = stack.pop() {
                (if v == 0.0 { first_chain } else { second_chain }).comm(stack)
            } else {
                Err(Error::StackEmpty(stack.len(), 1))
            },
            ConditionalKind::Try => match first_chain.comm(stack) {
                Err(_) => second_chain.comm(stack),
                Ok(v) => Ok(v)
            }
        }
        
    }
}

#[derive(Clone)]
pub struct Display(String);

impl CommandDesc for Display {
    const SHORT_NAME: Option<&'static str> = Some("Disp \"<Escaped String>\"");
    const NAME: &'static str = "Display \"<Escaped String>\"";
    const DESCRIPTION: &'static str = "Prints an escaped string to the command line. Only escaped characters are double quotes and backslashes, both escaped with a preceeding backslash.";
}

impl FromStr for Display {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! bail { () => {{return Err(Error::ParseToken(Self::NAME))}} }
        let trimmed_s = s.trim();
        if !s.to_uppercase().starts_with("DISPLAY") { bail!() }
        let mut new_s = trimmed_s.chars().skip(7).skip_while(|c| c.is_ascii_whitespace()).collect::<String>();
        if !new_s.ends_with('"') || new_s.ends_with("\\\"") { bail!() }
        new_s = new_s.chars().rev().skip(1).collect::<Vec<_>>().into_iter().rev().collect::<String>();
        if !new_s.starts_with('"') { bail!() }
        new_s = new_s.chars().skip(1).collect::<String>();
        let mut out = String::with_capacity(new_s.len());
        let mut new_s_chars = new_s.chars();
        while let Some(c) = new_s_chars.next() {
            match c {
                '\\' => if let Some(escaped_c) = new_s_chars.next() {
                    match escaped_c {
                        '\\' => out.push('\\'),
                        '"'  => out.push('"'),
                        _ => bail!()
                    }
                } else { bail!() },
                _ => out.push(c)
            }
        }
        Ok(Display(out))
    }
}

impl Command for Display {
    fn comm(self, _: &mut Vec<f64>) -> Result<Option<String>> {
        println!("{}",self.0);
        Ok(Some(self.0))
    }
}

#[derive(Clone)]
pub struct Input;

impl CommandDesc for Input {
    const SHORT_NAME: Option<&'static str> = Some("I");
    const NAME: &'static str = "Input";
    const DESCRIPTION: &'static str = "Allows the user to input a floating point number, and puts it on the stack. Mostly used in scripts.";
}

impl FromStr for Input {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_trim_up = s.trim().to_uppercase();
        match s_trim_up.as_str() {
            "I" | "INPUT" => Ok(Self),
            _ => Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl Command for Input {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        let mut buf = String::new();
        let stdin = std::io::stdin();
        let mut stdin_lock = stdin.lock();
        use std::io::BufRead;
        stdin_lock.read_line(&mut buf)?;
        Ok(buf.trim().parse::<ops::InsNum>()?.comm(stack)?)
    }
}

#[derive(Clone,PartialEq)]
pub struct Print;

impl CommandDesc for Print {
    const SHORT_NAME: Option<&'static str> = Some("P");
    const NAME: &'static str = "Print";
    const DESCRIPTION: &'static str = "Prints the top number in the stack to the screen. Mostly used in scripts.";
}

impl FromStr for Print {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "P" | "PRINT" => Ok(Print),
            _ => Err(Error::ParseToken(Self::NAME))
        }
    }
}

impl Command for Print {
    fn comm(self, stack: &mut Vec<f64>) -> Result<Option<String>> {
        let f_num = crate::format_num(stack.pop().ok_or(Error::StackEmpty(0, 1))?);
        println!("{f_num}");
        Ok(Some(f_num))
    }
}