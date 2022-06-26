use memorizer::text::{
    save_text_learnables, TextLearnable, TextRepresentation, TextTransform,
};
use memorizer::traits::{TransformId, RepresentationId, LearnableId};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut learnables = vec![];
    let mut id_base = 1655854159 << 16; // Totally legit unique number!
    let to_bin = TextTransform::new("binary to decimal", TransformId(id_base));
    id_base += 1;
    let to_dec = TextTransform::new("decimal to binary", TransformId(id_base));
    id_base += 1;

    for i in 0..16 {
        let i_dec = TextRepresentation::new(&format!("{i}"), RepresentationId(id_base));
        id_base += 1;
        let i_bin = TextRepresentation::new(&format!("{i:0>4b}"), RepresentationId(id_base));
        id_base += 1;
        learnables.push(TextLearnable::new(
            &[
                (i_dec.clone(), to_bin.clone(), i_bin.clone()),
                (i_bin.clone(), to_dec.clone(), i_dec.clone()),
            ],
            LearnableId(id_base),
        ));
        id_base += 1;
    }

    save_text_learnables(
        "/tmp/output.yaml",
        "Binary to Dec and vice versa",
        1655854159,
        &learnables,
    )?;
    Ok(())
}
