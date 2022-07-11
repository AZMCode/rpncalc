#![feature(trivial_bounds)]
#![feature(macro_metavar_expr)]
#![feature(array_chunks)]
#![feature(iter_intersperse)]

use std::io::Write;
use std::io::BufRead;

mod tui_error;
use tui_error::*;

mod input;

use rpncalc::Command;

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
            let stdin = std::io::stdin();
            let mut stdin_lock = stdin.lock();
            let stdout = std::io::stdout();
            let mut stdout_lock = stdout.lock();
            match rpncalc::Chain::from_bare(file_str)?.comm(&mut stack,&mut stdin_lock,&mut stdout_lock) {
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
                    Err(e.into())
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
                let mut input_buf = String::new();
                let stdin = std::io::stdin();
                let mut stdin_lock = stdin.lock();
                stdin_lock.read_line(&mut input_buf)?;
                use input::*;
                let input_res = input_buf.trim().parse::<Input>();
                match input_res {
                    Ok(input) => prev_msg_op = match input {
                        Input::Exit => break,
                        Input::Help => {
                            let [commands,ops] = [rpncalc::COMM_NAMES_DESCRIPTIONS.as_slice(),rpncalc::ops::OP_NAMES_DESCRIPTIONS.as_slice()].map(|t|
                                t.iter().fold(String::new(),|acc,(short_name_op,name,desc)|
                                    match short_name_op {
                                        Some(short_name) => format!("{acc} {short_name} | {name} : {desc}\n" ),
                                        None => format!("{acc} {name} : {desc}\n")
                                    }
                                )
                            );
                            Some(format!(include_str!("help_format_str.txt"),commands,ops))
                        }
                        Input::CommandOrOp(c) => {
                            prev_stack.clone_from(&stack);
                            match c.comm(&mut stack, &mut stdin_lock, &mut stdout_lock) {
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

fn print_stack(stack: &[f64], mut w: impl Write) -> Result {
    if stack.len() > 0 {
        for (index,elm) in stack.iter().rev().enumerate().rev() {
            let num_formatted = rpncalc::format_num(*elm);
            writeln!(w,"{:3}: {}",index,num_formatted)?;
        }
    } else {
        writeln!(w, "<Empty Stack>")?;
    }
    Ok(())
}