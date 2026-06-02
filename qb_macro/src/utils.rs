use darling::FromDeriveInput;
use syn::Type;

fn default_insert_returns() -> Type {
    syn::parse_quote!(())
}

fn default_primary_column() -> String {
    "id".to_string()
}

#[derive(FromDeriveInput)]
#[darling(attributes(model))]
pub struct ModelArgs {
    pub table_name: String,
    #[darling(default = "default_primary_column")]
    pub primary_column: String,
    #[darling(default = "default_insert_returns")]
    pub insert_returns: Type,
}