// PLECo

use std::collections::VecDeque;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Command(char),
    MultiLengthCommand(String),
    String(String),
    Var(String),
    Integer(i32),
    Expr(String),
    Code(String),
    Comment(String),
}

pub struct Lexer {
    pub input: VecDeque<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        while let Some(c) = self.input.pop_front() {
            match c {
                '#' => {
                    let mut arg = String::new();
                    while let Some(nc) = self.input.pop_front() {
                        if nc == '#' {
                            return Some(Token::Comment(arg));
                        } else {
                            arg.push(nc);
                        }
                    }
                    return Some(Token::Comment(arg));
                }
                '(' => {
                    let mut arg = String::new();
                    let mut escape_mode = false;
                    let mut depth = 1; // ネストの深さをカウント
                    while let Some(nc) = self.input.pop_front() {
                        if nc == '\\' && !escape_mode {
                            escape_mode = true;
                        } else if escape_mode {
                            match nc {
                                '\\' => arg.push('\\'),
                                'n' => arg.push('\n'),
                                _ => arg.push(nc),
                            }
                            escape_mode = false;
                        } else if nc == '(' {
                            depth += 1; // 入れ子が深くなる
                            arg.push(nc);
                        } else if nc == ')' {
                            depth -= 1; // 入れ子が浅くなる
                            if depth == 0 {
                                return Some(Token::Expr(arg));
                            }
                            arg.push(nc);
                        } else {
                            arg.push(nc);
                        }
                    }
                    return Some(Token::Expr(arg)); // `{` の対応が閉じていない場合
                }
                '*' => {
                    let mut arg = String::new();
                    while let Some(nc) = self.input.pop_front() {
                        if nc == ';' {
                            return Some(Token::Integer(arg.parse().unwrap_or(0)));
                        } else {
                            arg.push(nc);
                        }
                    }
                    return Some(Token::Integer(arg.parse().unwrap_or(0)));
                }
                '{' => {
                    let mut arg = String::new();
                    let mut escape_mode = false;
                    let mut depth = 1; // ネストの深さをカウント
                    while let Some(nc) = self.input.pop_front() {
                        if nc == '\\' && !escape_mode {
                            escape_mode = true;
                        } else if escape_mode {
                            match nc {
                                '\\' => arg.push('\\'),
                                'n' => arg.push('\n'),
                                _ => arg.push(nc),
                            }
                            escape_mode = false;
                        } else if nc == '{' {
                            depth += 1; // 入れ子が深くなる
                            arg.push(nc);
                        } else if nc == '}' {
                            depth -= 1; // 入れ子が浅くなる
                            if depth == 0 {
                                return Some(Token::Code(arg));
                            }
                            arg.push(nc);
                        } else {
                            arg.push(nc);
                        }
                    }
                    return Some(Token::Code(arg)); // `{` の対応が閉じていない場合
                }
                '^' => {
                    let mut arg = String::new();
                    while let Some(nc) = self.input.pop_front() {
                        if nc == '^' {
                            return Some(Token::MultiLengthCommand(arg));
                        } else {
                            arg.push(nc);
                        }
                    }
                    return Some(Token::MultiLengthCommand(arg));
                }
                '"' => {
                    let mut escape_mode = false;
                    let mut arg = String::new();
                    while let Some(nc) = self.input.pop_front() {
                        if nc == '\\' && !escape_mode {
                            escape_mode = true;
                        } else if escape_mode {
                            match nc {
                                '\\' => arg.push('\\'),
                                '"' => arg.push('"'),
                                'n' => arg.push('\n'),
                                _ => arg.push(nc),
                            }
                            escape_mode = false;
                        } else if nc == '"' {
                            return Some(Token::String(arg));
                        } else {
                            arg.push(nc);
                        }
                    }
                    return Some(Token::String(arg));
                }
                '$' => {
                    let mut arg = String::new();
                    while let Some(nc) = self.input.pop_front() {
                        if nc == '$' {
                            return Some(Token::Var(arg));
                        } else {
                            arg.push(nc);
                        }
                    }
                    return Some(Token::Var(arg));
                }
                _ => return Some(Token::Command(c)),
            }
        }
        None
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            if token != Token::Command(' ') && token != Token::Command('\n') {
                tokens.push(token);
            }
        }
        tokens
    }
}
