use syn::{
    braced, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Macro, Token,
};

use super::{try_parse, SqlDisplay};

#[derive(Debug)]
pub struct Schema {
    pub tables: Vec<TableDefinition>,
}

impl Parse for Schema {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tables = try_parse(input, |input| {
            let mut tables = Vec::new();

            while let Ok(m) = input.parse::<Macro>() {
                if let Some(id) = m.path.get_ident() {
                    if id == "table" {
                        let def: TableDefinition = m.parse_body()?;
                        tables.push(def);
                    }
                }

                let _ = input.parse::<Token![;]>();
            }

            Ok(tables)
        })?;

        Ok(Schema { tables })
    }
}

#[derive(Debug)]
pub struct TableDefinition {
    pub table: syn::Ident,
    pub keys_paren: Paren,
    pub keys: Punctuated<syn::Ident, Token![,]>,
    pub def_brace: Brace,
    pub def: Punctuated<FieldDefinition, Token![,]>,
}

impl Parse for TableDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            let keys;
            let def;

            Ok(Self {
                table: input.parse()?,
                keys_paren: parenthesized!(keys in input),
                keys: keys.parse_terminated(syn::Ident::parse)?,
                def_brace: braced!(def in input),
                def: def.parse_terminated(FieldDefinition::parse)?,
            })
        })
    }
}

#[derive(Debug)]
pub struct FieldDefinition {
    pub name: syn::Ident,
    pub arrow: (Token![-], Token![>]),
    pub ty: EitherNullableTypeOrIdentifier,
}

impl Parse for FieldDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(FieldDefinition {
            name: input.parse()?,
            arrow: (input.parse()?, input.parse()?),
            ty: input.parse()?,
        })
    }
}

type Identifier = super::Identifier;
impl_either!(pub, NullableType, Identifier);
impl_token_ident!(Nullable, "Nullable");

#[derive(Debug)]
pub struct NullableType {
    nullable: Nullable,
    pub lt: Token![<],
    ty: syn::Ident,
    pub gt: Token![>],
}

impl NullableType {
    pub fn get_type_name(&self) -> String {
        self.ty.to_string()
    }
}

impl Parse for NullableType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                nullable: input.parse()?,
                lt: input.parse()?,
                ty: input.parse()?,
                gt: input.parse()?,
            })
        })
    }
}

impl SqlDisplay for NullableType {
    fn fmt_sql(&self, _f: &mut std::fmt::Formatter<'_>, _level: usize) -> std::fmt::Result {
        Ok(())
    }
}
