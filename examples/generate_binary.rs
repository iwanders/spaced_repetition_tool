use memorizer_core::text::{
    save_text_learnables, TextLearnable, TextRepresentation, TextTransformation,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut learnables = vec![];
    let mut id_base = 1655854159 << 16; // Totally legit unique number!
    let to_bin = TextTransformation::new("binary to decimal", id_base);
    id_base += 1;
    let to_dec = TextTransformation::new("decimal to binary", id_base);
    id_base += 1;

    for i in 0..16 {
        let i_dec = TextRepresentation::new(&format!("{i}"), id_base);
        id_base += 1;
        let i_bin = TextRepresentation::new(&format!("{i:0>4b}"), id_base);
        id_base += 1;
        learnables.push(TextLearnable::new(
            &[
                (i_dec.clone(), to_bin.clone(), i_bin.clone()),
                (i_bin.clone(), to_dec.clone(), i_dec.clone()),
            ],
            id_base,
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
