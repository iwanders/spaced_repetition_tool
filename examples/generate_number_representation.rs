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
}

/// Program to generate a set of learnables, decks written to /tmp/
#[derive(Parser, Debug)]
#[clap(long_about = None)]
struct Args {
    /// The starting number (inclusive).
    min: u64,

    /// The end number (inclusive).
    max: u64,

    /// The directions to generate for each number in this range.
    #[clap(value_enum, required = true)]
    directions: Vec<Direction>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // We want all id's to be stable at all times, so here we make stable ids, shifting them by
    // 32 bits ought to be enough to cover anyones learning range?
    const DEC_SHIFT: u128 = 0xDEC << 32;
    const BINARY_SHIFT: u128 = 0xB0101 << 32;
    const HEX_SHIFT: u128 = 0x48E << 32; // 0x48 is H >_<
    const ASCII_SHIFT: u128 = 0xA5C11 << 32;

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
        ]);

    fn make_dec(value: u128) -> TextRepresentation {
        TextRepresentation::new(&format!("{value}"), RepresentationId(value + DEC_SHIFT))
    }
    fn make_bin(value: u128) -> TextRepresentation {
        TextRepresentation::new(
            &format!("{value:b}"),
            RepresentationId(value + BINARY_SHIFT),
        )
    }
    fn make_hex(value: u128) -> TextRepresentation {
        TextRepresentation::new(&format!("{value:x}"), RepresentationId(value + HEX_SHIFT))
    }
    fn make_ascii(value: u128) -> TextRepresentation {
        if !valid_ascii(value) {
            panic!("Requested ascii value for non valid number.");
        }
        let unprintables: std::collections::HashMap<u128, &str> =
            std::collections::HashMap::from([
                (0, "NUL"),
                (1, "SOH"),
                (2, "STX"),
                (3, "ETX"),
                (10, "LF"),
                (13, "CR"),
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
    fn valid_ascii(value: u128) -> bool {
        // Ok... so this is a bit tricky.
        // 0 to 32 are the unprintables, some of them are still useful, like \n and \r.
        // 32 itself is a space.
        if (value <= 4 || value == 10 || value == 13) || (value >= 32 && value <= 126) {
            return true;
        }
        false
    }

    let learnable_id_base: u128 = 1656462468 << 32; // Totally legit unique number!

    let args = Args::parse();

    let mut learnables = vec![];

    for i in args.min..=args.max {
        let v = i as u128;
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
    let min = args.min;
    let max = args.max;
    save_text_learnables(
        &format!("/tmp/deck_{min}_{max}_{z}.yaml"),
        &format!("{pretty} for {min} to {max}"),
        &learnables,
    )?;
    Ok(())
}
