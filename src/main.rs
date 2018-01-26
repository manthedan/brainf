#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]


extern crate brainf;

use std::mem;
use brainf::repl::Interpreter;
use brainf::repl::Parser;

#[allow(unused_assignments)]
fn main() {
    let mut input_buffer = String::new();
    let mut parser = Parser::new();
    let mut interpreter = Interpreter::new();

    println!("Starting BrainF REPL (type \"?\" to quit)");

    // Loop
    loop {
        // Read
        input_buffer = Parser::read_std();
        parser.tokenize(&input_buffer);

        // If `[` is unclosed continue accepting input
        while !parser.match_stack.is_empty() {
            input_buffer = Parser::read_cont();
            parser.tokenize(&input_buffer);
        }

        // Evaluate
        interpreter.take_tokens(mem::replace(&mut parser.tokens, Vec::new()));
        interpreter.interpret();

        parser.reset();

        // Print
        interpreter.print_brain();
    }
}
