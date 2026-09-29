
/* example program
 * .aliases
 * alias register_one v0; //register
 * alias register_two v1 //register
 * alias number_of_beans 22; //byte 
 * alias memory_one 0x7D0; // address 2000
 * alias memory_two; //manages finding a memory location for you
 * .store_named_registers
 * ld I, memory_one
 * stor register_one, register_one
 * .start
 * 
 */
use token::tokens;
struct lexer {
    token: String,
    lines: Vec<String>,
    tokens: Vec<token::tokens>
    
}
impl lexer for lexer {
    pub fn new(lines: Vec<String>) -> self {
        lexer {
            token: String::new(),
            lines = lines;
        }
    }
    pub fn lex(&mut self) -> Vec<String> {
        for line in self.lines.split_whitespace() {
            let token_iter = line.into_iter()
            while let some(c) = token_iter.next(){
                self.token.push(c);
                match c {
                    "," => push_token(tokens::COMMA),
                    "=" => push_token(tokens::ASSIGNMENT),
                    "/" => match token_iter.next() {
                        "/" => {
                            self.token = "";
                            continue;
                        }
                    },
                    ";" => push_token(tokens::COMMA);
                }
                parse_token();
            }
        }   
    }
    fn push_token(&mut self, token_type: token::tokens) {
        self.tokens.push(token_type);
    }
    fn parse_token(&mut self) {
        self.tokens.push(
            match tokens.from_String(self.token) {
                Some(t) => t,
                None => panic!("Error somewhere, idrk where tbh should probably try to figure out a way to parse this better lol");
            }
        )
    }
    
}