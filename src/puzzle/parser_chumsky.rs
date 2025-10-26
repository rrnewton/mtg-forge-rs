//! Chumsky-based parser for PZL files
//!
//! This is an experimental implementation using the chumsky parser combinator library
//! to evaluate error messages and compare approaches.

use chumsky::prelude::*;
use std::collections::HashMap;
use std::ops::Range;

/// Simple representation of a parsed PZL file
#[derive(Debug, Clone)]
pub struct ParsedPuzzle {
    pub metadata: ParsedMetadata,
    pub state: ParsedState,
}

#[derive(Debug, Clone)]
pub struct ParsedMetadata {
    pub name: String,
    pub goal: String,
    pub turns: u32,
    pub difficulty: String,
    pub url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedState {
    pub turn: u32,
    pub active_player: String,
    pub active_phase: String,
    pub p0_life: i32,
    pub p1_life: i32,
    pub p0_hand: Vec<String>,
    pub p1_hand: Vec<String>,
    pub p0_battlefield: Vec<String>,
    pub p1_battlefield: Vec<String>,
}

/// Parse a complete PZL file with chumsky
pub fn parse_puzzle(input: &str) -> std::result::Result<ParsedPuzzle, Vec<Simple<char>>> {
    puzzle_parser().parse(input)
}

/// Main parser for PZL file structure
fn puzzle_parser() -> impl Parser<char, ParsedPuzzle, Error = Simple<char>> {
    // Skip whitespace and comments
    let comment = just::<_, _, Simple<char>>('#')
        .then(take_until(text::newline()))
        .ignored();
    let ws_or_comment = choice((
        filter::<_, _, Simple<char>>(|c: &char| c.is_whitespace())
            .repeated()
            .at_least(1)
            .ignored(),
        comment,
    ))
    .repeated();

    // Section header: [name]
    let section_name = filter(|c: &char| c.is_alphanumeric() || *c == '_')
        .repeated()
        .at_least(1)
        .collect::<String>();

    let section_header = just('[')
        .ignore_then(section_name)
        .then_ignore(just(']'))
        .padded_by(ws_or_comment);

    // Key-value pair
    let key = filter(|c: &char| c.is_alphanumeric() || *c == '_')
        .repeated()
        .at_least(1)
        .collect::<String>();

    let value = none_of("\r\n")
        .repeated()
        .collect::<String>()
        .map(|s: String| s.trim().to_string());

    let kv_pair = key
        .then_ignore(just('=').or(just(':')))
        .then(value)
        .padded_by(ws_or_comment);

    // Section = header + pairs
    let section = section_header
        .then(kv_pair.repeated())
        .map(|(name, pairs)| (name, pairs));

    // Parse file as list of sections
    ws_or_comment
        .ignore_then(section.repeated().at_least(2))
        .then_ignore(ws_or_comment)
        .then_ignore(end())
        .try_map(|sections, span: Range<usize>| {
            let mut metadata_pairs = None;
            let mut state_pairs = None;

            for (name, pairs) in sections {
                match name.to_lowercase().as_str() {
                    "metadata" => metadata_pairs = Some(pairs),
                    "state" => state_pairs = Some(pairs),
                    _ => {}
                }
            }

            let metadata_pairs = metadata_pairs
                .ok_or_else(|| Simple::custom(span.clone(), "Missing [metadata] section"))?;
            let state_pairs =
                state_pairs.ok_or_else(|| Simple::custom(span, "Missing [state] section"))?;

            let metadata = parse_metadata(metadata_pairs)?;
            let state = parse_state(state_pairs)?;

            Ok(ParsedPuzzle { metadata, state })
        })
}

fn parse_metadata(pairs: Vec<(String, String)>) -> Result<ParsedMetadata, Simple<char>> {
    let map: HashMap<String, String> = pairs.into_iter().collect();

    let name = map
        .get("Name")
        .ok_or_else(|| Simple::custom(0..0, "Missing required field 'Name' in [metadata] section"))?
        .clone();

    let goal = map
        .get("Goal")
        .cloned()
        .unwrap_or_else(|| "Win".to_string());
    let turns = map.get("Turns").and_then(|s| s.parse().ok()).unwrap_or(1);

    let difficulty = map
        .get("Difficulty")
        .cloned()
        .unwrap_or_else(|| "Easy".to_string());

    Ok(ParsedMetadata {
        name,
        goal,
        turns,
        difficulty,
        url: map.get("URL").cloned(),
        description: map.get("Description").cloned(),
    })
}

fn parse_state(pairs: Vec<(String, String)>) -> Result<ParsedState, Simple<char>> {
    let map: HashMap<String, String> = pairs.into_iter().collect();

    let turn = map
        .get("turn")
        .or_else(|| map.get("Turn"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let active_player = map
        .get("activeplayer")
        .or_else(|| map.get("ActivePlayer"))
        .cloned()
        .unwrap_or_else(|| "p0".to_string());

    let active_phase = map
        .get("activephase")
        .or_else(|| map.get("ActivePhase"))
        .cloned()
        .unwrap_or_else(|| "MAIN1".to_string());

    let p0_life = map
        .get("p0life")
        .or_else(|| map.get("humanlife"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let p1_life = map
        .get("p1life")
        .or_else(|| map.get("ailife"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let parse_cards = |s: &str| -> Vec<String> {
        if s.trim().is_empty() {
            Vec::new()
        } else {
            s.split(';')
                .map(|card| card.trim().to_string())
                .filter(|card| !card.is_empty())
                .collect()
        }
    };

    let p0_hand = map
        .get("p0hand")
        .or_else(|| map.get("humanhand"))
        .map(|s| parse_cards(s))
        .unwrap_or_default();
    let p1_hand = map
        .get("p1hand")
        .or_else(|| map.get("aihand"))
        .map(|s| parse_cards(s))
        .unwrap_or_default();
    let p0_battlefield = map
        .get("p0battlefield")
        .or_else(|| map.get("humanbattlefield"))
        .map(|s| parse_cards(s))
        .unwrap_or_default();
    let p1_battlefield = map
        .get("p1battlefield")
        .or_else(|| map.get("aibattlefield"))
        .map(|s| parse_cards(s))
        .unwrap_or_default();

    Ok(ParsedState {
        turn,
        active_player,
        active_phase,
        p0_life,
        p1_life,
        p0_hand,
        p1_hand,
        p0_battlefield,
        p1_battlefield,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_parse_simple_puzzle() {
        let input = r#"
[metadata]
Name=Test Puzzle
Goal=Win
Turns=1
Difficulty=Easy

[state]
turn=1
activeplayer=p0
activephase=MAIN1
p0life=20
p1life=10
"#;

        let result = parse_puzzle(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let puzzle = result.unwrap();
        assert_eq!(puzzle.metadata.name, "Test Puzzle");
        assert_eq!(puzzle.state.turn, 1);
        assert_eq!(puzzle.state.p0_life, 20);
        assert_eq!(puzzle.state.p1_life, 10);
    }

    #[test]
    fn test_parse_with_cards() {
        let input = r#"
[metadata]
Name=Card Test
Goal=Win
Turns=1
Difficulty=Medium

[state]
turn=1
activeplayer=p0
activephase=MAIN1
p0life=20
p0hand=Lightning Bolt;Grizzly Bears
p0battlefield=Mountain;Mountain;Forest
p1life=20
p1battlefield=Plains
"#;

        let result = parse_puzzle(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let puzzle = result.unwrap();
        assert_eq!(puzzle.state.p0_hand.len(), 2);
        assert_eq!(puzzle.state.p0_hand[0], "Lightning Bolt");
        assert_eq!(puzzle.state.p0_battlefield.len(), 3);
        assert_eq!(puzzle.state.p1_battlefield.len(), 1);
    }

    #[test]
    fn test_error_missing_name() {
        let input = r#"
[metadata]
Goal=Win
Turns=1
Difficulty=Easy

[state]
turn=1
"#;

        let result = parse_puzzle(input);
        assert!(result.is_err());

        if let Err(errors) = result {
            println!("\n=== Chumsky Error for Missing Name ===");
            for error in &errors {
                println!("{:?}", error);
            }
        }
    }

    #[test]
    fn test_difficulty_defaults_to_easy() {
        let input = r#"
[metadata]
Name=Test
Goal=Win
Turns=1

[state]
turn=1
"#;

        let result = parse_puzzle(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let puzzle = result.unwrap();
        assert_eq!(puzzle.metadata.difficulty, "Easy");
    }

    #[test]
    fn test_error_missing_metadata_section() {
        let input = r#"
[state]
turn=1
"#;

        let result = parse_puzzle(input);
        assert!(result.is_err());

        if let Err(errors) = result {
            println!("\n=== Chumsky Error for Missing Metadata Section ===");
            for error in &errors {
                println!("{:?}", error);
            }
        }
    }

    #[test]
    fn test_parse_sample_puzzles() {
        let puzzle_dir = PathBuf::from("forge-java/forge-gui/res/puzzle");

        if !puzzle_dir.exists() {
            println!("Skipping: puzzle directory not found");
            return;
        }

        let mut parsed_count = 0;
        let mut failed_count = 0;
        let mut failures = Vec::new();

        for entry in fs::read_dir(&puzzle_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("pzl") {
                let contents = fs::read_to_string(&path).unwrap();
                match parse_puzzle(&contents) {
                    Ok(_) => parsed_count += 1,
                    Err(e) => {
                        failed_count += 1;
                        let filename = path.file_name().unwrap().to_string_lossy();
                        failures.push(format!("{}: {:?}", filename, e.first()));
                    }
                }

                // Test all files for full evaluation
                // if parsed_count + failed_count >= 20 {
                //     break;
                // }
            }
        }

        println!("\n=== Chumsky Parser Sample Results ===");
        println!("Parsed: {}/{}", parsed_count, parsed_count + failed_count);

        if !failures.is_empty() {
            println!("\nFirst few failures:");
            for failure in failures.iter().take(5) {
                println!("  {}", failure);
            }
        }
    }
}
