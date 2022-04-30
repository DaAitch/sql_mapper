macro_rules! impl_token_ident {
    ($name:ident, $string:expr) => {
        #[derive(Debug)]
        struct $name;

        impl Parse for $name {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let ident: syn::Ident = input.parse()?;

                if ident == $string {
                    Ok(Self)
                } else {
                    Err(syn::Error::new(ident.span(), $string))
                }
            }
        }

        impl $crate::parse::SqlDisplay for $name {
            fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, _: usize) -> std::fmt::Result {
                write!(f, $string)
            }
        }
    };
}

macro_rules! impl_token_punct {
    ($name:ident, $punct:tt) => {
        #[derive(Debug)]
        struct $name;

        impl Parse for $name {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let _: syn::Token![$punct] = input.parse()?;

                Ok(Self)
            }
        }

        impl $crate::parse::SqlDisplay for $name {
            fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, _: usize) -> std::fmt::Result {
                write!(f, stringify! {$punct})
            }
        }
    };
}

macro_rules! impl_either {
    ($a:ident, $b:ident) => { impl_either! { , $a, $b } };
    ($v:vis, $a:ident, $b:ident) => {
        paste::paste! {
            #[derive(Debug)]
            $v enum [< Either $a Or $b >] {
                $a($a),
                $b($b),
            }

            impl Parse for [< Either $a Or $b >] {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    try_parse(input, |input| {
                        if let Ok(first) = input.parse::<$a>() {
                            return Ok(Self::$a(first));
                        }

                        let second = input.parse::<$b>()?;
                        Ok(Self::$b(second))
                    })
                }
            }

            impl $crate::parse::SqlDisplay for [< Either $a Or $b >] {
                fn fmt_sql(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
                    match self {
                        Self::$a(inner) => $crate::parse::SqlDisplay::fmt_sql(inner, f, level),
                        Self::$b(inner) => $crate::parse::SqlDisplay::fmt_sql(inner, f, level),
                    }
                }
            }
        }
    };
}
