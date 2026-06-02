mod utils;

use crate::utils::ModelArgs;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(Model, attributes(model))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let args = match ModelArgs::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    
    let ident = &input.ident;
    let table_name = to_snake_case(&args.table_name);
    let primary_column = args.primary_column;
    let insert_returns = args.insert_returns;
    
    let expanded = quote! {
        impl Model for #ident {
            const TABLE_NAME: &'static str = #table_name;
            const PRIMARY_COLUMN: &'static str = #primary_column;

            type InsertReturns = #insert_returns;
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
