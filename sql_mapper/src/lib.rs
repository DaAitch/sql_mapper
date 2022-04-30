#![feature(proc_macro_span)]

#[macro_use]
mod r#macro;
mod parse;

use parse::Sql;
use proc_macro::TokenStream;
use syn::parse_macro_input;

// https://docs.rs/syn/latest/syn/parse/index.html
// https://teiid.github.io/teiid-documents/9.0.x/content/reference/BNF_for_SQL_Grammar.html

#[proc_macro]
pub fn sql(item: TokenStream) -> TokenStream {
    let output = parse_macro_input!(item as Sql);
    output.into()
}
