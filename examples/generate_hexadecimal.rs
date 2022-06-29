use memorizer::text::{save_text_learnables, TextLearnable, TextRepresentation, TextTransform};
use memorizer::traits::{LearnableId, RepresentationId, TransformId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut learnables = vec![];
    let mut id_base = 1656462468 << 16; // Totally legit unique number!
    let to_hex = TextTransform::new("decimal to hexadecimal", TransformId(id_base));
    id_base += 1;
    let to_dec = TextTransform::new("hexadecimal to decimal", TransformId(id_base));
    id_base += 1;

    let max_value = 16;
    for i in 0..max_value {
        let i_dec = TextRepresentation::new(&format!("{i}"), RepresentationId(id_base));
        id_base += 1;
        let i_hex = TextRepresentation::new(&format!("{i:x}"), RepresentationId(id_base));
        id_base += 1;
        learnables.push(TextLearnable::new(
            &[
                (i_dec.clone(), to_hex.clone(), i_hex.clone()),
                (i_hex.clone(), to_dec.clone(), i_dec.clone()),
            ],
            LearnableId(id_base),
        ));
        id_base += 1;
    }

    save_text_learnables(
        &format!("/tmp/hexadecimal_decimal_to_{max_value}.yaml"),
        "Binary to Dec and vice versa",
        1656462468,
        &learnables,
    )?;
    Ok(())
}
