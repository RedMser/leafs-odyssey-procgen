use std::str::FromStr;

use leafs_odyssey_data::data::*;
use logos::{Lexer, Logos};

#[derive(Logos)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[regex(r"[a-zA-Z]+")]
    Text,
    #[regex(r"\d+")]
    Number,
}

#[derive(Clone)]
pub enum Item {
    Tile(String),
    Stack(Vec<Item>),
}

impl From<&Item> for LOTile {
    fn from(item: &Item) -> Self {
        match item {
            Item::Tile(tile) => {
                LOTile::from_str(&tile).unwrap_or(LOTile::None)
            },
            Item::Stack(stack) => {
                LOTile::Stack {
                    tiles: stack.iter().map(|i| LOStackElement {
                        direction: LOStackDirection::default(),
                        connections: vec![],
                        tile: i.into(),
                    }).collect()
                }
            },
        }
    }
}

impl From<&Item> for LOStackTile {
    fn from(item: &Item) -> Self {
        match item {
            Item::Tile(tile) => {
                LOStackTile::from_str(&tile).unwrap_or(LOStackTile::None)
            },
            Item::Stack(_) => {
                LOStackTile::None
            },
        }
    }
}

pub fn parse_string_to_items(input: &str) -> Result<Vec<Item>, String> {
    let mut lexer = Token::lexer(input);
    let items = parse(&mut lexer, false)?;

    for item in items.iter() {
        if let Item::Stack(stack) = item {
            if stack.len() > 16 {
                return Err(format!("Stacks can only contain up to 16 elements, but found one that had {}.", stack.len()).to_string());
            }
        }
    }

    Ok(items)
}

fn parse(lexer: &mut Lexer<Token>, in_stack: bool) -> Result<Vec<Item>, String> {
    let mut items = vec![];

    loop {
        let token = lexer.next();
        if token.is_none() {
            break;
        }

        let token = token.unwrap().unwrap();
        match token {
            Token::Text => {
                items.push(Item::Tile(lexer.slice().to_owned()));
            },
            Token::OpenParen => {
                items.push(Item::Stack(parse(lexer, true)?));
            },
            Token::CloseParen => {
                if in_stack {
                    if items.is_empty() {
                        return Err("An empty item stack was specified \"()\".".to_string());
                    }
                    break;
                } else {
                    return Err("Closing parenthesis without opening counterpart.".to_string());
                }
            },
            Token::Plus => {
                // For now everything is treated as a plus. Might have other handling in the future...
            },
            Token::Star => {
                let token = lexer.next();
                if token.is_none() {
                    return Err("String ended with a star but no number.".to_string());
                }
                let token = token.unwrap().unwrap();
                if let Token::Number = token {
                    let repeat_count = lexer.slice().parse::<i32>().map_err(|_| "Star was not followed by a valid number".to_string())?;
                    if repeat_count <= 0 {
                        return Err("Star repetition count must be positive.".to_string());
                    } else if repeat_count > 2000 {
                        return Err("Star repetition count is unreasonably high.".to_string());
                    }

                    let to_repeat = items.last();
                    if to_repeat.is_none() {
                        return Err("Star found but is not preceded by anything repeatable.".to_string());
                    }
                    let to_repeat = to_repeat.unwrap().clone();

                    for _ in 0..(repeat_count-1) {
                        items.push(to_repeat.clone());
                    }
                } else {
                    return Err("Star was not followed by a number.".to_string());
                }
            }
            Token::Number => {
                return Err("Unexpected number on toplevel.".to_string());
            },
        }
    }

    Ok(items)
}