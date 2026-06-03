mod utils;

use crate::utils::{ModelArgs, ModelInsertArgs};
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
    let table_name = utils::to_snake_case(&args.table_name);
    let primary_column = args.primary_column;

    let expanded = quote! {
        impl<'q, DB, E> Model<'q, DB, E> for #ident
        where
            DB: Database,
            E: Executor<'q, Database = DB> + Clone,
            DB::Arguments: IntoArguments<DB>,
            String: sqlx::Encode<'q, DB>,
            String: sqlx::Type<DB>,
        {
            const TABLE_NAME: &'static str = #table_name;
            const PRIMARY_COLUMN: &'static str = #primary_column;
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ModelInsert, attributes(model))]
pub fn model_insert_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let args = match ModelInsertArgs::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let ident = &input.ident;
    let table_name = args.table_name.map(|s| utils::to_snake_case(&s));
    let table_name = table_name.as_deref();

    let insert_returns = args.insert_returns;

    let expanded = quote! {
        impl<'q, DB, E> ModelInsert<'q, #insert_returns> for #ident
        {
            const TABLE_NAME: &'static str = #table_name;
        }
    };

    TokenStream::from(expanded)
}
