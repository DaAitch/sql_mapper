use super::{
    schema::{EitherNullableTypeOrIdentifier, Schema},
    try_parse, Identifier, SqlDisplay,
};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

// walker

pub trait ProjectionWalker {
    fn walk_projection(&self, schema: &Schema, f: &mut dyn FnMut(&Projection));
}

#[derive(Debug)]
pub struct Projection<'a> {
    pub name: String,
    pub diesel_type: &'a EitherNullableTypeOrIdentifier,
}

// parsable structs

///
///
///
#[derive(Debug)]
pub struct SqlStatement {
    pub file: syn::LitStr,
    pub name: syn::Ident,
    query: Query,
}

impl Parse for SqlStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![-]>()?;
        input.parse::<Token![-]>()?;

        Ok(SqlStatement {
            file: input.parse()?,
            name: input.parse()?,
            query: input.parse()?,
        })
    }
}

impl ProjectionWalker for SqlStatement {
    fn walk_projection(&self, schema: &Schema, f: &mut dyn FnMut(&Projection)) {
        self.query.walk_projection(schema, f)
    }
}

impl Display for SqlStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        SqlDisplay::fmt_sql(&self.query, f, 0)
    }
}

///
///
///
#[derive(Debug)]
struct Query {
    select_clause: SelectClause,
    into_clause: Option<IntoClause>,
    rest: Option<(
        FromClause,
        Option<()>, // where
        Option<()>, // group by
        Option<()>, // having
    )>,
}

impl Parse for Query {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                select_clause: input.parse()?,
                into_clause: input.parse().ok(),
                rest: try_parse(input, |input| Ok((input.parse()?, None, None, None))).ok(),
            })
        })
    }
}

impl ProjectionWalker for Query {
    fn walk_projection(&self, schema: &Schema, f: &mut dyn FnMut(&Projection)) {
        match &self.select_clause.star_or_sublists {
            EitherStarOrPunctSelectSublist::Star(_) => {
                let (from, _, _, _) = self.rest.as_ref().expect("* with not table is not valid");
                for tr in &from.table_references.0 {
                    let name = tr
                        .joined_table
                        .table_primary
                        .table_name
                        .identifier
                        .id
                        .to_string();
                    if let Some(table) = schema.tables.iter().find(|t| t.table == name) {
                        for field in &table.def {
                            let field_name = field.name.to_string();
                            f(&Projection {
                                name: format!("{}.{}", name, field_name),
                                diesel_type: &field.ty,
                            })
                        }
                    } else {
                        panic!("cannot find table {}", name);
                    }
                }
            }
            EitherStarOrPunctSelectSublist::PunctSelectSublist(_) => {}
        }
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        SqlDisplay::fmt_sql(self, f, 0)
    }
}

impl SqlDisplay for Query {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        indent(f, level)?;
        self.select_clause.fmt_sql(f, level)?;

        if let Some(into) = &self.into_clause {
            writeln!(f)?;
            indent(f, level)?;
            into.fmt_sql(f, level)?;
        }

        if let Some((from, _, _, _)) = &self.rest {
            writeln!(f)?;
            indent(f, level)?;
            from.fmt_sql(f, level)?;
        }

        Ok(())
    }
}

///
///
///
#[derive(Debug)]
struct SelectClause {
    select: Select,
    all_or_distinct: Option<EitherAllOrDistinct>,
    star_or_sublists: EitherStarOrPunctSelectSublist,
}

impl Parse for SelectClause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                select: input.parse()?,
                all_or_distinct: input.parse().ok(),
                star_or_sublists: input.parse()?,
            })
        })
    }
}

impl SqlDisplay for SelectClause {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        self.select.fmt_sql(f, level)?;

        if let Some(v) = &self.all_or_distinct {
            write!(f, " ")?;
            v.fmt_sql(f, level)?;
        }

        {
            let level = level + 1;
            writeln!(f)?;
            indent(f, level)?;
            self.star_or_sublists.fmt_sql(f, level)?;
        }

        Ok(())
    }
}

///
///
///
#[derive(Debug)]
struct SelectSublist;

impl Parse for SelectSublist {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        panic!("{:?}", input);
    }
}

impl SqlDisplay for SelectSublist {
    fn fmt_sql(&self, _f: &mut std::fmt::Formatter<'_>, _level: usize) -> std::fmt::Result {
        Ok(())
    }
}

///
///
///
#[derive(Debug)]
struct IntoClause {
    into: Into,
    identifier: Identifier,
}

impl Parse for IntoClause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                into: input.parse()?,
                identifier: input.parse()?,
            })
        })
    }
}

impl SqlDisplay for IntoClause {
    fn fmt_sql(&self, _f: &mut std::fmt::Formatter<'_>, _level: usize) -> std::fmt::Result {
        todo!()
    }
}

///
///
/// https://teiid.github.io/teiid-documents/9.0.x/content/reference/BNF_for_SQL_Grammar.html#from
#[derive(Debug)]
struct FromClause {
    from: From,
    table_references: Punctuated<TableReference, Comma, OneOreMany>,
}

impl Parse for FromClause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                from: input.parse()?,
                table_references: input.parse()?,
            })
        })
    }
}

impl SqlDisplay for FromClause {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        self.from.fmt_sql(f, level)?;

        {
            writeln!(f)?;

            let level = level + 1;
            indent(f, level)?;
            self.table_references.fmt_sql(f, level)?;
        }

        Ok(())
    }
}

///
///
/// https://teiid.github.io/teiid-documents/9.0.x/content/reference/BNF_for_SQL_Grammar.html#tableReference
#[derive(Debug)]
struct TableReference {
    joined_table: JoinedTable,
}

impl Parse for TableReference {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                joined_table: input.parse()?,
            })
        })
    }
}

impl SqlDisplay for TableReference {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        self.joined_table.fmt_sql(f, level)
    }
}

///
///
/// https://teiid.github.io/teiid-documents/9.0.x/content/reference/BNF_for_SQL_Grammar.html#joinedTable
#[derive(Debug)]
struct JoinedTable {
    table_primary: TablePrimary,
    // todo cross_join ...
}

impl Parse for JoinedTable {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                table_primary: input.parse()?,
            })
        })
    }
}

impl SqlDisplay for JoinedTable {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        self.table_primary.fmt_sql(f, level)
    }
}

///
///
/// https://teiid.github.io/teiid-documents/9.0.x/content/reference/BNF_for_SQL_Grammar.html#tablePrimary
#[derive(Debug)]
struct TablePrimary {
    table_name: TableName,
}

impl Parse for TablePrimary {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                table_name: input.parse()?,
            })
        })
    }
}

impl SqlDisplay for TablePrimary {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        self.table_name.fmt_sql(f, level)
    }
}

///
///
/// https://teiid.github.io/teiid-documents/9.0.x/content/reference/BNF_for_SQL_Grammar.html#unaryFromClause
#[derive(Debug)]
struct TableName {
    identifier: Identifier,
    as_: Option<(Option<As>, Identifier)>,
}

impl Parse for TableName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| {
            Ok(Self {
                identifier: input.parse()?,
                as_: try_parse(input, |input| Ok((input.parse().ok(), input.parse()?))).ok(),
            })
        })
    }
}

impl SqlDisplay for TableName {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        self.identifier.fmt_sql(f, level)?;

        if let Some((as_, id)) = &self.as_ {
            if let Some(as_) = as_ {
                write!(f, " ")?;
                as_.fmt_sql(f, level)?;
            }

            id.fmt_sql(f, level)?;
        }

        Ok(())
    }
}

impl_token_ident!(Select, "SELECT");
impl_token_ident!(All, "ALL");
impl_token_ident!(Distinct, "DISTINCT");
impl_token_ident!(Into, "INTO");
impl_token_ident!(From, "FROM");
impl_token_ident!(As, "AS");

impl_token_punct!(Star, *);
impl_token_punct!(Comma, ,);

type PunctSelectSublist = Punctuated<SelectSublist, Comma>;
impl_either!(Star, PunctSelectSublist);
impl_either!(All, Distinct);

// tokens & helper structs

struct Punctuated<T, P, S = NoneOrMany>(syn::punctuated::Punctuated<T, P>, PhantomData<S>);

impl<T: Parse, P: Parse, S: PunctuationStrategy<T, P>> Parse for Punctuated<T, P, S> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        try_parse(input, |input| match S::parse(input) {
            Ok(p) => Ok(Self(p, PhantomData)),
            Err(err) => Err(err),
        })
    }
}

impl<T: Debug, P: Debug, S> std::fmt::Debug for Punctuated<T, P, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.pairs().map(|e| match e {
                syn::punctuated::Pair::Punctuated(a, b) => {
                    format!("{:?} {:?}", a, b)
                }
                syn::punctuated::Pair::End(v) => {
                    format!("{:?}", v)
                }
            }))
            .finish()
    }
}

impl<T: SqlDisplay, P: SqlDisplay, S> SqlDisplay for Punctuated<T, P, S> {
    fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        for (value, punct) in self.0.pairs().map(syn::punctuated::Pair::into_tuple) {
            value.fmt_sql(f, level)?;
            if let Some(punct) = punct {
                punct.fmt_sql(f, level)?;
            }
        }

        Ok(())
    }
}

trait PunctuationStrategy<T, P> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<syn::punctuated::Punctuated<T, P>>;
}

struct NoneOrMany;

impl<T: Parse, P: Parse> PunctuationStrategy<T, P> for NoneOrMany {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<syn::punctuated::Punctuated<T, P>> {
        syn::punctuated::Punctuated::parse_terminated(input)
    }
}

struct OneOreMany;

impl<T: Parse, P: Parse> PunctuationStrategy<T, P> for OneOreMany {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<syn::punctuated::Punctuated<T, P>> {
        let result: syn::punctuated::Punctuated<T, P> =
            syn::punctuated::Punctuated::parse_terminated(input)?;

        if result.is_empty() {
            Err(input.error("empty not allowed"))
        } else {
            Ok(result)
        }
    }
}

// helper

fn indent(f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
    let ident = 4 * level;
    write!(f, "{:ident$}", "")
}
