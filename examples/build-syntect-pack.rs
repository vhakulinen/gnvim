#[cfg(feature = "libwebkit2gtk")]
extern crate syntect;

fn main() {
    #[cfg(feature = "libwebkit2gtk")]
    {
        use syntect::dumps::*;
        use syntect::parsing::SyntaxSetBuilder;

        let mut builder = SyntaxSetBuilder::new();
        builder.add_plain_text_syntax();
        builder
            .add_from_folder("./sublime-syntaxes/syntaxes/", true)
            .unwrap();
        let ss = builder.build();
        dump_to_file(&ss, "./sublime-syntaxes/all.pack").unwrap();
    }
}
