pub mod repl {
    use std::io;
    use std::io::prelude::*;
    use std::process;
    use std::fmt;

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
    fn read_input(prompt: Prompt) -> String {
        print!("{}  ", char_from_prompt(prompt));
        io::stdout().flush().expect("failed to flush prompt buffer");

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        line.trim().to_string()
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
    #[derive(Copy, Clone, Debug)]
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
        prev_cursor: usize,
    }

    impl Parser {
        pub fn new() -> Parser {
            Parser {
                tokens: Vec::new(),
                match_stack: Vec::new(),
                cursor: 0,
                prev_cursor: 0,
            }
        }

        pub fn read_std() -> String {
            read_input(Prompt::Input)
        }

        pub fn read_cont() -> String {
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
                    '[' => if self.push_match(Token::JumpForward(0)).is_err() {
                        return;
                    },
                    ']' => if self.push_match(Token::JumpBackward(0)).is_err() {
                        return;
                    },
                    '?' => process::exit(0),
                    _ => (),
                }
            }
        }

        fn push_token(&mut self, token: Token) {
            self.tokens.push(token);
            self.cursor += 1;
        }

        fn push_match(&mut self, token: Token) -> Result<(), ()> {
            match token {
                Token::JumpForward(_) => {
                    // TODO: Figure this out
                    let cursor = self.cursor;
                    self.match_stack.push(cursor);
                    self.push_token(Token::JumpForward(0));
                }
                Token::JumpBackward(_) => {
                    let prev = self.match_stack.pop();
                    match prev {
                        None => {
                            self.error();
                            return Err(());
                        }
                        Some(i) => {
                            let prev_cursor = self.prev_cursor;
                            self.tokens[i] = Token::JumpForward(self.cursor + prev_cursor);
                            self.push_token(Token::JumpBackward(i + prev_cursor));
                        }
                    }
                }
                _ => (),
            }
            Ok(())
        }

        fn error(&mut self) {
            println!("{}  Unbalanced ']' input", char_from_prompt(Prompt::Error));
            self.reset();
        }

        pub fn reset(&mut self) {
            self.tokens = Vec::new();
            self.match_stack = Vec::new();
            self.prev_cursor += self.cursor;
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

        pub fn take_tokens(&mut self, mut tokens: Vec<Token>) {
            self.tokens.append(&mut tokens);
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
            self.cursor = i - 1;
        }
    }

    // The data cells and cell pointer
    // output_buffer makes the output operator a little easier
    #[derive(Default)]
    pub struct Brain {
        cells: Vec<u8>,
        ptr: usize,
        output_buffer: String,
    }

    impl Brain {
        fn new() -> Brain {
            Brain {
                cells: vec![0; 1],
                ptr: 0,
                output_buffer: String::new(),
            }
        }

        fn read_byte(&self) -> String {
            read_input(Prompt::Byte)
        }

        fn flush_output_buffer(&mut self) {
            if !self.output_buffer.is_empty() {
                println!("{}", self.output_buffer);
                self.output_buffer.clear();
            }
        }

        fn input(&mut self) {
            // I don't know if this is good or bad
            if let Some(n) = self.read_byte().chars().next() {
                self.add(n as u8)
            }
        }

        fn output(&mut self) {
            self.output_buffer.push(self.cells[self.ptr] as char);
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
            let output: String = self.cells
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
}
