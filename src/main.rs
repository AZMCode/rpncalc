#![feature(trivial_bounds)]
#![feature(macro_metavar_expr)]
#![feature(array_chunks)]
#![feature(iter_intersperse)]

use std::io::Write;
use std::io::BufRead;

mod error;
mod input;

use error::*;

use crate::input::Command;

#[derive(clap::Parser)]
#[clap(version,about,author)]
struct Main {
    /// Executes a file instead of opening the REPL. Format of the file is a UTF-8 'chain' command as shown in the REPL, except no need to type the enclosing square brackets.
    file: Option<std::path::PathBuf>
}

fn main() -> Result {
    use clap::Parser;
    let Main {
        file
    } = Main::parse();
    let mut stack = vec![];
    match file {
        Some(p) => {
            let file_bytes = std::fs::read(p)?;
            let file_str = std::str::from_utf8(&file_bytes)?;
            match input::Chain::from_bare(file_str)?.comm(&mut stack) {
                Ok(msg_opt) => {
                    match msg_opt {
                        Some(msg) => println!("{msg}"),
                        None => ()
                    }
                    println!("Exited successfully with the following stack:");
                    let stdout = std::io::stdout();
                    let mut stdout_lock = stdout.lock();
                    print_stack(&stack, &mut stdout_lock)?;
                    Ok(())
                },
                Err(e) => {
                    println!("Program ended with error with the following stack:");
                    let stdout = std::io::stdout();
                    let mut stdout_lock = stdout.lock();
                    print_stack(&stack, &mut stdout_lock)?;
                    std::mem::drop(stdout_lock);
                    println!("And with the following error:");
                    Err(e)
                }
            }
        },
        None => {
            let mut prev_stack = vec![];
            let mut prev_msg_op = Some("Type 'h' or 'help' for a list of commands".to_string());
            loop {
                clearscreen::clear()?;
                let stdout = std::io::stdout();
                let mut stdout_lock = stdout.lock();
                if let Some(prev_msg) = prev_msg_op {
                    writeln!(stdout_lock,"{}",prev_msg)?;
                }
                print_stack(&stack,&mut stdout_lock)?;
                write!(stdout_lock, "> ")?;
                stdout_lock.flush()?;
                std::mem::drop(stdout_lock);
                let mut input_buf = String::new();
                let stdin = std::io::stdin();
                let mut stdin_lock = stdin.lock();
                stdin_lock.read_line(&mut input_buf)?;
                std::mem::drop(stdin_lock);
                use input::*;
                let input_res = input_buf.trim().parse::<Input>();
                match input_res {
                    Ok(input) => prev_msg_op = match input {
                        Input::Exit => break,
                        Input::Command(c) => {
                            prev_stack.clone_from(&stack);
                            match c.comm(&mut stack) {
                                Ok(new_msg_op) => 
                                    new_msg_op,
                                Err(e) => {
                                    stack.clone_from(&prev_stack);
                                    Some(format!("Error executing last command, reverting stack: \n{0}",e))
                                },
                            }
                        },
                        Input::Op(o) => {
                            prev_stack.clone_from(&stack);
                            match o.comm(&mut stack) {
                                Ok(new_msg_op) => 
                                    new_msg_op,
                                Err(e) => {
                                    stack.clone_from(&prev_stack);
                                    Some(format!("Error executing last command, reverting stack: \n{0}",e))
                                },
                            }
                        }
                    },
                    Err(e) => prev_msg_op = Some(format!("Could not parse last command: \n{0}",e))
                }
            }
            clearscreen::clear()?;
            {
                let stdout = std::io::stdout();
                let mut stdout_lock = stdout.lock();
                writeln!(stdout_lock,"Exiting Successfully with the following stack:")?;
                print_stack(&stack, &mut stdout_lock)?;
            }
            Ok(())
        }
    }
}

pub fn format_num(input: f64) -> String {
    let sign = if input.is_sign_positive() { " " } else {"-"};
    if input.is_finite() {
        let abs_num = input.abs();
        let left_digits_len = if abs_num <= 1.0 { 1 } else { abs_num.log10() as usize + 1 };
        let excess_digits = left_digits_len % 3;
        let padding_amount = if excess_digits == 0 { 0 } else { 3 - excess_digits };
        let padding = ' '.to_string().repeat(padding_amount);
        let num = format!("{padding}{abs_num:.9}");
        let regrouped_num = num.split('.').map(|side| side.chars()
            .fold((vec![],vec![]),|(mut tail,mut curr),c| {
                curr.push(c);
                if curr.len() == 3 {
                    tail.push(curr);
                    (tail,vec![])
                } else {
                    (tail,curr)
                }
            }).0.into_iter().map(|three_chunk|
                three_chunk.into_iter().fold(String::with_capacity(3),|mut acc,c| { acc.push(c); acc })
            ).intersperse(" ".to_string()).fold(String::new(),|acc,s| acc + &s)
        ).intersperse(".".to_string()).fold(String::new(),|acc,s| acc + &s);
        format!("{sign}{regrouped_num}")
    } else {
        if input.is_nan() {
            format!("{sign}NaN")
        } else {
            format!("{sign}Inf")
        }
    }
}

fn print_stack(stack: &[f64], mut w: impl Write) -> Result {
    if stack.len() > 0 {
        for (index,elm) in stack.iter().rev().enumerate().rev() {
            let num_formatted = format_num(*elm);
            writeln!(w,"{:3}: {}",index,num_formatted)?;
        }
    } else {
        writeln!(w, "<Empty Stack>")?;
    }
    Ok(())
}