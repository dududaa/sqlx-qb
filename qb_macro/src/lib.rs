use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, Lit, parse_macro_input};

#[proc_macro_derive(QbModel, attributes(model))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let ident = &input.ident;

    let mut table_name = ident.to_string().to_lowercase();
    for attr in &input.attrs {
        if attr.path().is_ident("model") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table_name") {
                    let value = meta.value()?;
                    if let Expr::Lit(expr_lit) = value.parse::<Expr>()? {
                        if let Lit::Str(lit_str) = expr_lit.lit {
                            table_name = lit_str.value();
                        }
                    }
                }

                Ok(())
            });
        }
    }

    let expanded = quote! {
        impl Model for #ident {
            const TABLE_NAME: &'static str = #table_name;
        }
    };

    TokenStream::from(expanded)
}
