extern crate syntect;

use syntect::parsing::SyntaxSetBuilder;
use syntect::dumps::*;

fn main() {
    let mut builder = SyntaxSetBuilder::new();
    builder.add_plain_text_syntax();
    builder.add_from_folder("./sublime-syntaxes/syntaxes/", true).unwrap();
    let ss = builder.build();
    dump_to_file(&ss, "./sublime-syntaxes/all.pack").unwrap();
}
