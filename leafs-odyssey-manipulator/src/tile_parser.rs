use std::str::FromStr;

use leafs_odyssey_data::data::*;
use logos::{Lexer, Logos};

#[derive(Logos)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token("+")]
    Plus,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[regex(r"\*\d+")]
    Multiply,
    #[regex(r"[a-zA-Z:0-9,]+")]
    Text,
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
                if tile.contains(':') {
                    let mut parts = tile.split(':');
                    let mut base_tile = LOTile::from_str(parts.next().unwrap()).unwrap_or(LOTile::None);
                    match base_tile {
                        LOTile::StartPoint { ref mut direction } |
                        LOTile::BombBug { ref mut direction } |
                        LOTile::FlyingSnake { ref mut direction } |
                        LOTile::Slug { ref mut direction } => {
                            *direction = LODirection::from_str(parts.next().unwrap()).expect("Invalid direction string.");
                        },
                        LOTile::ToggleSwitch { ref mut connections } |
                        LOTile::SacrificeAltar { ref mut connections } |
                        LOTile::PressurePlate { ref mut connections } => {
                            while let Some(part) = parts.next() {
                                let (x, y) = part.split_once(',').expect("Invalid connection target string.");
                                let (x, y) = (
                                    x.parse().expect("Connection target X position is not an integer."),
                                    y.parse().expect("Connection target Y position is not an integer."),
                                );
                                connections.push(LOConnection { x_position: x, y_position: y });
                            }
                        }
                        _ => {},
                    }
                    base_tile
                } else {
                    LOTile::from_str(&tile).unwrap_or(LOTile::None)
                }
            },
            Item::Stack(stack) => {
                LOTile::Stack {
                    tiles: stack.iter().map(|i| LOTile::from(i).into()).collect()
                }
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
            Token::Multiply => {
                let repeat_count = lexer.slice().strip_prefix('*').unwrap().parse::<i32>().map_err(|_| "Star was not followed by a valid number".to_string())?;
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
            }
        }
    }

    Ok(items)
}