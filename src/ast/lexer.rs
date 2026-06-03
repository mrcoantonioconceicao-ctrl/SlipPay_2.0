#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    Number(f64),
    Boolean(bool),

    And,     // &&
    Or,      // ||
    Equal,   // ==
    Less,    // <
    Greater, // >

    LParen, // (
    RParen, // )
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\n' | '\t' => {
                chars.next();
            }

            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::And);
                }
            }

            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::Or);
                }
            }

            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Equal);
                }
            }

            '<' => {
                chars.next();
                tokens.push(Token::Less);
            }

            '>' => {
                chars.next();
                tokens.push(Token::Greater);
            }

            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }

            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }

            '0'..='9' => {
                let mut num = String::new();

                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let value = num.parse::<f64>().unwrap();
                tokens.push(Token::Number(value));
            }

            _ if ch.is_alphabetic() || ch == '_' => {
                let mut ident = String::new();

                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                match ident.as_str() {
                    "true" => tokens.push(Token::Boolean(true)),
                    "false" => tokens.push(Token::Boolean(false)),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }

            _ => {
                chars.next();
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_rule() {
        let input = "amount < 5000 && risk_score > 40";
        let tokens = tokenize(input);

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("amount".into()),
                Token::Less,
                Token::Number(5000.0),
                Token::And,
                Token::Identifier("risk_score".into()),
                Token::Greater,
                Token::Number(40.0),
            ]
        );
    }
}
