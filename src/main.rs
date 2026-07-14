use brainf::repl::Interpreter;
use brainf::repl::Parser;

fn main() {
    let mut parser = Parser::new();
    let mut interpreter = Interpreter::new();

    println!("Starting BrainF REPL (type \"?\" to quit)");

    // Loop
    'repl: while let Some(input) = Parser::read_std() {
        if input.trim() == "?" {
            break;
        }

        // Read
        parser.tokenize(&input);

        // If `[` is unclosed continue accepting input
        while !parser.match_stack.is_empty() {
            let Some(input) = Parser::read_cont() else {
                eprintln!("🚨  Unbalanced '[' input");
                return;
            };
            if input.trim() == "?" {
                break 'repl;
            }
            parser.tokenize(&input);
        }

        // Evaluate
        interpreter.take_tokens(std::mem::take(&mut parser.tokens));
        interpreter.interpret();

        parser.reset();

        // Print
        interpreter.print_brain();
    }
}
