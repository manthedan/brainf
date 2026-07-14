pub mod repl {
    use std::fmt;
    use std::io;
    use std::io::prelude::*;

    // Enums for shell prompt symbols
    #[derive(Copy, Clone, Debug)]
    enum Prompt {
        Input,
        Continue,
        Byte,
        State,
        Error,
    }

    // Print shell prompt then accept user input
    fn read_input(prompt: Prompt) -> Option<String> {
        print!("{}  ", char_from_prompt(prompt));
        io::stdout().flush().expect("failed to flush prompt buffer");

        let mut line = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut line)
            .expect("failed to read input");

        if bytes_read == 0 {
            None
        } else {
            Some(line.trim_end_matches(['\r', '\n']).to_string())
        }
    }

    // Returns symbols defined for prompt
    // Using Emoji to be annoying
    fn char_from_prompt(prompt: Prompt) -> char {
        match prompt {
            Prompt::Input => '👉',
            Prompt::Continue => '💦',
            Prompt::Byte => '🍴',
            Prompt::State => '🙏',
            Prompt::Error => '🚨',
        }
    }

    // Tokens that compromise our language
    // Usize is used to index the Jump tokens
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Token {
        PointerIncrement,
        PointerDecrement,
        DataIncrement,
        DataDecrement,
        Input,
        Output,
        JumpForward(usize),
        JumpBackward(usize),
    }

    // Parser to tokenize
    #[derive(Default, Debug)]
    pub struct Parser {
        pub tokens: Vec<Token>,
        pub match_stack: Vec<usize>,
        cursor: usize,
    }

    impl Parser {
        pub fn new() -> Parser {
            Parser {
                tokens: Vec::new(),
                match_stack: Vec::new(),
                cursor: 0,
            }
        }

        pub fn read_std() -> Option<String> {
            read_input(Prompt::Input)
        }

        pub fn read_cont() -> Option<String> {
            read_input(Prompt::Continue)
        }

        pub fn tokenize(&mut self, input: &str) {
            for n in input.chars() {
                match n {
                    '>' => self.push_token(Token::PointerIncrement),
                    '<' => self.push_token(Token::PointerDecrement),
                    '+' => self.push_token(Token::DataIncrement),
                    '-' => self.push_token(Token::DataDecrement),
                    '.' => self.push_token(Token::Output),
                    ',' => self.push_token(Token::Input),
                    '[' => self.push_open(),
                    ']' if self.push_close().is_err() => return,
                    ']' => {}
                    _ => (),
                }
            }
        }

        fn push_token(&mut self, token: Token) {
            self.tokens.push(token);
            self.cursor += 1;
        }

        fn push_open(&mut self) {
            self.match_stack.push(self.cursor);
            self.push_token(Token::JumpForward(0));
        }

        fn push_close(&mut self) -> Result<(), ()> {
            let Some(open) = self.match_stack.pop() else {
                self.error();
                return Err(());
            };

            self.tokens[open] = Token::JumpForward(self.cursor);
            self.push_token(Token::JumpBackward(open));
            Ok(())
        }

        fn error(&mut self) {
            println!("{}  Unbalanced ']' input", char_from_prompt(Prompt::Error));
            self.reset();
        }

        pub fn reset(&mut self) {
            self.tokens = Vec::new();
            self.match_stack = Vec::new();
            self.cursor = 0;
        }
    }

    // Interpreter reads tokens and executes their instructions
    #[derive(Default)]
    pub struct Interpreter {
        pub brain: Brain,
        tokens: Vec<Token>,
        cursor: usize,
    }

    impl Interpreter {
        pub fn new() -> Interpreter {
            Interpreter {
                brain: Brain::new(),
                tokens: Vec::new(),
                cursor: 0,
            }
        }

        // Printing the memory cell state as a REPL feature
        pub fn print_brain(&self) {
            println!("{} {}", char_from_prompt(Prompt::State), self.brain);
        }

        pub fn take_tokens(&mut self, tokens: Vec<Token>) {
            self.tokens = tokens;
            self.cursor = 0;
        }

        pub fn interpret(&mut self) {
            while self.cursor < self.tokens.len() {
                let cursor = self.cursor;
                match self.tokens[cursor] {
                    Token::PointerIncrement => self.brain.ptr_right(),
                    Token::PointerDecrement => self.brain.ptr_left(),
                    Token::DataIncrement => self.brain.increment(),
                    Token::DataDecrement => self.brain.decrement(),
                    Token::Output => self.brain.output(),
                    Token::Input => self.brain.input(),
                    Token::JumpForward(i) => self.forward(i),
                    Token::JumpBackward(i) => self.backward(i),
                }
                self.cursor += 1;
            }
            self.brain.flush_output_buffer();
        }

        fn forward(&mut self, i: usize) {
            if self.brain.is_zero() {
                self.cursor = i;
            }
        }

        fn backward(&mut self, i: usize) {
            if !self.brain.is_zero() {
                self.cursor = i;
            }
        }
    }

    // The data cells and cell pointer
    // output_buffer makes the output operator a little easier
    #[derive(Default)]
    pub struct Brain {
        cells: Vec<u8>,
        ptr: usize,
        output_buffer: Vec<u8>,
    }

    impl Brain {
        fn new() -> Brain {
            Brain {
                cells: vec![0; 1],
                ptr: 0,
                output_buffer: Vec::new(),
            }
        }

        fn read_byte(&self) -> Option<String> {
            read_input(Prompt::Byte)
        }

        fn flush_output_buffer(&mut self) {
            if !self.output_buffer.is_empty() {
                let mut stdout = io::stdout().lock();
                stdout
                    .write_all(&self.output_buffer)
                    .and_then(|()| stdout.write_all(b"\n"))
                    .expect("failed to write output");
                self.output_buffer.clear();
            }
        }

        fn input(&mut self) {
            if let Some(byte) = self
                .read_byte()
                .as_deref()
                .and_then(|input| input.as_bytes().first())
            {
                self.cells[self.ptr] = *byte;
            }
        }

        fn output(&mut self) {
            self.output_buffer.push(self.cells[self.ptr]);
        }

        fn ptr_right(&mut self) {
            self.ptr += 1;
            if self.ptr > self.cells.len() - 1 {
                self.cells.push(0);
            }
        }

        fn ptr_left(&mut self) {
            if self.ptr == 0 {
                return;
            }
            self.ptr -= 1;
        }

        fn increment(&mut self) {
            self.add(1)
        }

        fn decrement(&mut self) {
            self.cells[self.ptr] = self.cells[self.ptr].wrapping_sub(1);
        }

        fn add(&mut self, n: u8) {
            self.cells[self.ptr] = self.cells[self.ptr].wrapping_add(n);
        }

        fn is_zero(&self) -> bool {
            self.cells[self.ptr] == 0
        }
    }

    // Custom display to indicate current memory state
    impl fmt::Display for Brain {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let output: String = self
                .cells
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    if self.ptr == i {
                        format!(" [{}]", cell)
                    } else {
                        format!(" {}", cell)
                    }
                })
                .collect();
            write!(f, "{}", output)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn run(interpreter: &mut Interpreter, source: &str) {
            let mut parser = Parser::new();
            parser.tokenize(source);
            assert!(parser.match_stack.is_empty(), "test program is unbalanced");
            interpreter.take_tokens(parser.tokens);
            interpreter.interpret();
        }

        #[test]
        fn parser_links_nested_loops() {
            let mut parser = Parser::new();
            parser.tokenize("[[]]");

            assert_eq!(
                parser.tokens,
                vec![
                    Token::JumpForward(3),
                    Token::JumpForward(2),
                    Token::JumpBackward(1),
                    Token::JumpBackward(0),
                ]
            );
        }

        #[test]
        fn parser_treats_question_marks_as_comments() {
            let mut parser = Parser::new();
            parser.tokenize("?+?");

            assert_eq!(parser.tokens, vec![Token::DataIncrement]);
        }

        #[test]
        fn loops_can_start_a_new_repl_entry() {
            let mut interpreter = Interpreter::new();
            run(&mut interpreter, "+");
            run(&mut interpreter, "[-]");

            assert_eq!(interpreter.brain.cells, vec![0]);
        }

        #[test]
        fn rejected_input_does_not_corrupt_the_next_program() {
            let mut parser = Parser::new();
            parser.tokenize("+]");
            assert!(parser.tokens.is_empty());

            parser.tokenize("+[-]");
            let mut interpreter = Interpreter::new();
            interpreter.take_tokens(parser.tokens);
            interpreter.interpret();

            assert_eq!(interpreter.brain.cells, vec![0]);
        }

        #[test]
        fn nested_loops_work() {
            let mut interpreter = Interpreter::new();
            run(&mut interpreter, "++[>++[>+<-]<-]");

            assert_eq!(interpreter.brain.cells, vec![0, 0, 4]);
            assert_eq!(interpreter.brain.ptr, 0);
        }

        #[test]
        fn output_is_byte_oriented() {
            let mut brain = Brain::new();
            brain.decrement();
            brain.output();

            assert_eq!(brain.output_buffer, vec![255]);
        }
    }
}
