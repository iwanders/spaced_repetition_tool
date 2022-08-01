use memorizer::text::{save_text_learnables, TextLearnable, TextRepresentation, TextTransform};
use memorizer::traits::{LearnableId, RepresentationId, TransformId};

use clap::{Parser, ValueEnum};

#[derive(Debug, ValueEnum, Clone, Eq, PartialEq, Hash)]
enum Direction {
    DecBin,
    BinDec,
    HexDec,
    DecHex,
    AsciiDec, // really functions as utf8, but who wants to write emoji's?
    DecAscii,
    AsciiHex,
    HexAscii,
}

/// Program to generate a set of learnables, decks written to /tmp/
#[derive(Parser, Debug)]
#[clap(long_about = None)]
struct Args {
    /// The output directory.
    #[clap(short, long)]
    output_dir: Option<String>,

    /// The number specification, "0..=3,4,0b101" gives (0, 1, 2, 3, 4, 5), "0x10..=0x20" gives
    /// (16 to 32 (inclusive)).
    number_spec: String,

    /// The directions to generate for each number in this range.
    #[clap(value_enum, required = true)]
    directions: Vec<Direction>,
}

/// Convert a single number.
fn parse_number(v: &str) -> u64 {
    use std::str::FromStr;
    if v.starts_with("0x") {
        return u64::from_str_radix(v.trim_start_matches("0x"), 16).expect("unable to parse hex");
    }
    if v.starts_with("0b") {
        return u64::from_str_radix(v.trim_start_matches("0b"), 2).expect("unable to parse binary");
    }

    return u64::from_str(v).expect("unable to parse binary");
}

/// Convert a string of number specifications.
fn parse_number_spec(v: &str) -> Vec<u64> {
    let mut res: Vec<u64> = vec![];
    for r in v.split(",") {
        let mut start_end = r.split("..");
        if start_end.clone().count() == 1 {
            res.push(parse_number(start_end.next().expect("Must have a value")));
            continue;
        }

        let start = start_end
            .next()
            .expect("Should've had a start of the range");
        let end_string = start_end.next().expect("Should've had a end of the range");

        let start = parse_number(&start);
        let end: u64;
        if end_string.starts_with("=") {
            end = parse_number(&end_string[1..]) + 1;
        } else {
            end = parse_number(&end_string);
        }
        for i in start..end {
            res.push(i);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_input() {
        assert_eq!(parse_number(&"30"), 30);
        assert_eq!(parse_number(&"0x30"), 48);
        assert_eq!(parse_number(&"0b1101"), 13);

        assert_eq!(parse_number_spec("0..3"), Vec::<u64>::from([0, 1, 2]));
        assert_eq!(parse_number_spec("0..3,4"), Vec::<u64>::from([0, 1, 2, 4]));
        assert_eq!(
            parse_number_spec("0..=3,4"),
            Vec::<u64>::from([0, 1, 2, 3, 4])
        );
        assert_eq!(
            parse_number_spec("0..=3,4,0b101"),
            Vec::<u64>::from([0, 1, 2, 3, 4, 5])
        );
        assert_eq!(
            parse_number_spec("0..=3,4,0x20"),
            Vec::<u64>::from([0, 1, 2, 3, 4, 32])
        );
        assert_eq!(
            parse_number_spec("0x10..=0x15,0b010..=0b100"),
            Vec::<u64>::from([16, 17, 18, 19, 20, 21, 2, 3, 4])
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // We want all id's to be stable at all times, so here we make stable ids, shifting them by
    // 32 bits ought to be enough to cover anyones learning range?
    const DEC_SHIFT: u64 = 0xDEC << 32;
    const BINARY_SHIFT: u64 = 0xB0101 << 32;
    const HEX_SHIFT: u64 = 0x48E << 32; // 0x48 is H >_<
    const ASCII_SHIFT: u64 = 0xA5C11 << 32;

    // Make a map of the transforms.
    let transforms: std::collections::HashMap<Direction, TextTransform> =
        std::collections::HashMap::from([
            (
                Direction::DecBin,
                TextTransform::new("decimal to binary", TransformId(0xD2B << 32)),
            ),
            (
                Direction::BinDec,
                TextTransform::new("binary to decimal", TransformId(0xB2D << 32)),
            ),
            (
                Direction::HexDec,
                TextTransform::new("hexadecimal to decimal", TransformId(0x482D << 32)),
            ),
            (
                Direction::DecHex,
                TextTransform::new("decimal to hexadecimal", TransformId(0xD248 << 32)),
            ),
            (
                Direction::AsciiDec,
                TextTransform::new("ascii to decimal", TransformId(0xA5C112B << 32)),
            ),
            (
                Direction::DecAscii,
                TextTransform::new("decimal to ascii", TransformId(0xB2A5C11 << 32)),
            ),
            (
                Direction::AsciiHex,
                TextTransform::new("ascii to hexadecimal", TransformId(0xA5C112482D << 32)),
            ),
            (
                Direction::HexAscii,
                TextTransform::new("hexadecimal to ascii", TransformId(0x482D2A5C11 << 32)),
            ),
        ]);

    fn make_dec(value: u64) -> TextRepresentation {
        TextRepresentation::new(&format!("{value}"), RepresentationId(value + DEC_SHIFT))
    }
    fn make_bin(value: u64) -> TextRepresentation {
        TextRepresentation::new(
            &format!("{value:b}"),
            RepresentationId(value + BINARY_SHIFT),
        )
    }
    fn make_hex(value: u64) -> TextRepresentation {
        TextRepresentation::new(&format!("{value:x}"), RepresentationId(value + HEX_SHIFT))
    }
    fn make_ascii(value: u64) -> TextRepresentation {
        if !valid_ascii(value) {
            panic!("Requested ascii value for non valid number.");
        }
        let unprintables: std::collections::HashMap<u64, &str> = std::collections::HashMap::from([
            (0, "NUL"),
            (1, "SOH"),
            (2, "STX"),
            (3, "ETX"),
            (4, "EOT"),
            (5, "ENQ"),
            (6, "ACK"),
            (7, "BEL"),
            (8, "BS"),
            (9, "HT"),
            (10, "LF"),
            (11, "VT"),
            (12, "FF"),
            (13, "CR"),
            (14, "SO"),
            (15, "SI"),
            (16, "DLE"),
            (17, "DC1"),
            (18, "DC2"),
            (19, "DC3"),
            (20, "DC4"),
            (21, "NAK"),
            (22, "SYN"),
            (23, "ETB"),
            (24, "CAN"),
            (25, "EM"),
            (26, "SUB"),
            (27, "ESC"),
            (28, "FS"),
            (29, "GS"),
            (30, "RS"),
            (31, "US"),
            (127, "DEL"),
        ]);
        if let Some(v) = unprintables.get(&value) {
            return TextRepresentation::new(v, RepresentationId(value + ASCII_SHIFT));
        }
        return TextRepresentation::new(
            &format!(
                "{}",
                std::char::from_u32(value as u32).expect("Should be valid ascii")
            ),
            RepresentationId(value + ASCII_SHIFT),
        );
    }
    fn valid_ascii(value: u64) -> bool {
        // Ok... so this is a bit tricky.
        // 0 to 32 are the unprintables, some of them are still useful, like \n and \r.
        // 32 itself is a space.
        // if (value <= 4 || value == 10 || value == 13) || (value >= 32 && value <= 126) {
        // return true;
        // }
        // false
        value <= 127 // everything below 127 is now valid ascii.
    }

    let learnable_id_base: u64 = 1656462468 << 32; // Totally legit unique number!

    let args = Args::parse();

    let mut learnables = vec![];

    let numbers = parse_number_spec(&args.number_spec);

    for i in numbers {
        let v = i as u64;
        let mut edges = vec![];
        for direction in args.directions.iter() {
            match direction {
                m if m == &Direction::BinDec => {
                    edges.push((
                        make_bin(v),
                        transforms.get(&m).unwrap().clone(),
                        make_dec(v),
                    ));
                }
                m if m == &Direction::DecBin => {
                    edges.push((
                        make_dec(v),
                        transforms.get(&m).unwrap().clone(),
                        make_bin(v),
                    ));
                }
                m if m == &Direction::HexDec => {
                    edges.push((
                        make_hex(v),
                        transforms.get(&m).unwrap().clone(),
                        make_dec(v),
                    ));
                }
                m if m == &Direction::DecHex => {
                    edges.push((
                        make_dec(v),
                        transforms.get(&m).unwrap().clone(),
                        make_hex(v),
                    ));
                }
                m if m == &Direction::AsciiDec => {
                    if valid_ascii(v) {
                        edges.push((
                            make_ascii(v),
                            transforms.get(&m).unwrap().clone(),
                            make_dec(v),
                        ));
                    }
                }
                m if m == &Direction::DecAscii => {
                    if valid_ascii(v) {
                        edges.push((
                            make_dec(v),
                            transforms.get(&m).unwrap().clone(),
                            make_ascii(v),
                        ));
                    }
                }
                m if m == &Direction::AsciiHex => {
                    if valid_ascii(v) {
                        edges.push((
                            make_ascii(v),
                            transforms.get(&m).unwrap().clone(),
                            make_hex(v),
                        ));
                    }
                }
                m if m == &Direction::HexAscii => {
                    if valid_ascii(v) {
                        edges.push((
                            make_hex(v),
                            transforms.get(&m).unwrap().clone(),
                            make_ascii(v),
                        ));
                    }
                }
                _ => {}
            }
        }
        learnables.push(TextLearnable::new(
            &(edges[..]),
            LearnableId(learnable_id_base + v),
        ));
    }

    let directions = args
        .directions
        .iter()
        .map(|z| format!("{z:?}"))
        .collect::<Vec<String>>();
    let z = directions.clone().join("_");
    let pretty = directions.clone().join(", ");

    let numbers = args.number_spec;
    save_text_learnables(
        &(args.output_dir.unwrap_or("/tmp".to_owned()) + &format!("/deck_{numbers}_{z}.yaml")),
        &format!("{pretty} for {numbers}"),
        &learnables,
    )?;
    Ok(())
}
