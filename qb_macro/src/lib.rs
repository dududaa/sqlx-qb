use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, Lit, parse_macro_input};

#[proc_macro_derive(QbModel, attributes(model))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let ident = &input.ident;

    let mut table_name = to_snake_case(&ident.to_string());
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

fn to_snake_case(name: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            snake_case.push('_');
        }

        snake_case.push(c.to_ascii_lowercase());
    }

    snake_case
}

#[cfg(test)]
mod tests {
    use crate::to_snake_case;

    #[test]
    fn test_snake_case() {
        assert_eq!("foo_bar".to_string(), to_snake_case("FooBar"));
    }
}
