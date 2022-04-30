use self::{
    schema::{EitherNullableTypeOrIdentifier, Schema},
    sql::SqlStatement,
};
use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse},
};

mod schema;
mod sql;

use sql::ProjectionWalker;

// SQL Query Builder

pub trait SqlDisplay {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result;
}

pub struct Sql {
    schema: Schema,
    statement: SqlStatement,
}

impl Parse for Sql {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let statement: SqlStatement = input.parse()?;

        let path = Span::call_site().source_file().path();
        let path = path.parent().unwrap();
        let path = path.join(statement.file.value());

        let schema = match std::fs::read_to_string(path) {
            Ok(string) => string,
            Err(err) => {
                return Err(syn::Error::new_spanned(
                    &statement.file,
                    format!("cannot read file: {:?}", err),
                ));
            }
        };

        let stream: TokenStream = schema.parse().unwrap();
        let schema: Schema = syn::parse_macro_input::parse(stream)?;

        Ok(Sql { schema, statement })
    }
}

fn sql_name_to_id(name: &String) -> String {
    name.chars()
        .into_iter()
        .map(|c| if c == '.' { '_' } else { c })
        .collect()
}

fn sql_type_to_ty(name: &String) -> String {
    match name.as_str() {
        "Int4" => "i32",
        "Text" => "String",
        "Float8" => "f32",
        x => panic!("unknown type: {}", x),
    }
    .to_owned()
}

impl std::convert::Into<TokenStream> for Sql {
    fn into(self) -> TokenStream {
        let s = format!("{}", self.statement);
        let name = &self.statement.name;

        let mut fields = Vec::new();
        let mut mapper = Vec::new();
        let mut i = 0usize;

        self.statement.walk_projection(&self.schema, &mut |proj| {
            let name = sql_name_to_id(&proj.name);

            let ty = match proj.diesel_type {
                EitherNullableTypeOrIdentifier::NullableType(ty) => {
                    let t = sql_type_to_ty(&(ty.get_type_name()));
                    let t = format_ident!("{}", t);
                    quote! {
                        ::std::option::Option<#t>
                    }
                }
                EitherNullableTypeOrIdentifier::Identifier(ty) => {
                    let t = sql_type_to_ty(&(ty.id.to_string()));
                    let t = format_ident!("{}", t);
                    quote! {
                        #t
                    }
                }
            };

            let name = format_ident!("{}", name);

            fields.push(quote! {
                pub #name: #ty
            });

            mapper.push(quote! {
                #name: row.get(#i)
            });

            i += 1;
        });

        quote! {
            #[doc = " SQL:"]
            #[doc = ""]
            #[doc = "```sql"]
            #[doc = #s]
            #[doc = "```"]
            #[derive(Debug)]
            pub struct #name {
                #(#fields),*
            }

            impl #name {
                pub fn query(client: &mut ::postgres::Client) -> ::std::result::Result<Vec<#name>, ::postgres::Error> {
                    let result = client.query(#s, &[])?;
                    let items: Vec<_> = result.iter().map(|row| {
                        #name {
                            #(#mapper),*
                        }
                    }).collect();

                    Ok(items)
                }
            }
      }.into()
    }
}

//

///
///
///
#[derive(Debug)]
struct Identifier {
    id: syn::Ident,
}

impl Parse for Identifier {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            id: syn::Ident::parse_any(input)?,
        })
    }
}

impl SqlDisplay for Identifier {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, _: usize) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

// helper

fn try_parse<T>(
    input: syn::parse::ParseStream,
    f: impl FnOnce(syn::parse::ParseStream) -> syn::Result<T>,
) -> syn::Result<T> {
    let fork = input.fork();

    let result = f(&fork);
    if result.is_ok() {
        input.advance_to(&fork);
    }

    result
}

mod debug {
    use std::fmt::Debug;
    use syn::parse::{Parse, ParseStream};

    fn die(x: &dyn Debug) -> ! {
        panic!("{:#?}", x);
    }

    fn debug_parse<T: Parse>(input: ParseStream) -> syn::Result<T> {
        let v = input.parse::<T>();
        if let Err(err) = v {
            panic!("error: {:?} before {:?}", err, input)
        }

        v
    }
}
