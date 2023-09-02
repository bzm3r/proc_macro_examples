use quote::quote;
use std::collections::HashMap;
use syn::{
    parse_macro_input, parse_quote, token::Pound, Attribute, Expr, Fields, ItemStruct, MetaList,
    Token,
};

// Function like proc_macros must have signature (TokenStream) -> TokenStream.
// In this case, we assume that the function will take as input a well defined
// Rust struct. For example:
//         #[derive(Parser)]
//         struct Foo {
//             #[arg(long, env = "BAR")]
//             bar: String
//             #[arg(long, env = "BAZ")]
//             baz: usize,
//         }
#[proc_macro]
pub fn gen_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // see: https://docs.rs/syn/latest/syn/struct.ItemStruct.html
    let ItemStruct {
        attrs,
        vis,
        struct_token,
        ident,
        generics,
        fields,
        semi_token,
    } = parse_macro_input!(input as ItemStruct);

    // parse_quote (https://docs.rs/syn/latest/syn/macro.parse_quote.html) lets
    // us easily convert something we know can be parsed as an expression into a
    // syn::Expr
    let default_expr_map: HashMap<&'static str, Expr> = HashMap::from([
        ("bar", parse_quote!(String::from("bar"))),
        ("baz", parse_quote!(0)),
    ]);

    let final_struct = ItemStruct {
        attrs,
        vis,
        struct_token,
        ident,
        generics,
        fields: define_default_vals(default_expr_map, fields),
        semi_token,
    };

    let tokens = quote! {
        #final_struct
    };

    eprintln!("output tokens:\n{:#?}", tokens.to_string());
    tokens.into()
}

fn define_default_vals(
    default_expr_map: HashMap<&'static str, Expr>,
    mut fields: Fields,
) -> Fields {
    // see the docs for syn::Fields: https://docs.rs/syn/latest/syn/enum.Fields.html
    match &mut fields {
        Fields::Named(named_fields) => {
            for named in named_fields.named.iter_mut() {
                let default_expr = default_expr_map.get(
                    named
                        .ident
                        .as_ref()
                        .expect("A named field must have a name!")
                        .to_string()
                        .as_str(),
                ).expect("Currently, the macro does not know what to do if there is no default expr for a field.");

                // see the docs for syn::Attribute: https://docs.rs/syn/latest/syn/struct.Attribute.html
                if let Some(args_meta_list) = find_args_attr(named.attrs.iter_mut()) {
                    args_meta_list.tokens = if !args_meta_list.tokens.is_empty() {
                        let existing_tokens = args_meta_list.tokens.clone();
                        quote!(#existing_tokens, default_value_t = #default_expr)
                    } else {
                        let pound: Pound = parse_quote!(#);
                        quote!(#pound[arg(default_value_t = #default_expr)])
                    };
                } else {
                    named
                        .attrs
                        .push(parse_quote!(#[args(default_value_t = #default_expr)]))
                }
            }
        }
        _ => panic!("This macro does not yet know how to deal with fields that are not named!"),
    }
    fields
}

fn find_args_attr<'a>(
    mut attrs: impl Iterator<Item = &'a mut Attribute>,
) -> Option<&'a mut MetaList> {
    attrs.find_map(|attr| match &mut attr.meta {
        syn::Meta::List(meta_list)
            if meta_list
                .path
                .get_ident()
                .map(|id| id == "arg")
                .unwrap_or(false) =>
        {
            Some(meta_list)
        }
        _ => None,
    })
}
