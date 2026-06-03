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
}

#[derive(FromDeriveInput)]
#[darling(attributes(model))]
pub struct ModelInsertArgs {
    pub table_name: Option<String>,
    #[darling(default = "default_insert_returns")]
    pub insert_returns: Type
}

pub fn to_snake_case(name: &str) -> String {
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
    use crate::utils::to_snake_case;

    #[test]
    fn test_snake_case() {
        assert_eq!("foo_bar".to_string(), to_snake_case("FooBar"));
    }
}