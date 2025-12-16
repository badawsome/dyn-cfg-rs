use proc_macro::TokenStream;
use quote::quote;
mod watcher;
use syn::{Data, DeriveInput, Expr, Field, Fields, Type, parse_macro_input};

struct FieldConfig {
    field_name: syn::Ident,
    field_type: Type,
    key: String,
    parse_fn: syn::Expr,
    default_value: Option<syn::Expr>,
    is_required: bool,
}

#[proc_macro_attribute]
pub fn dynamic_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let struct_name = &input.ident;
    let vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let mut field_configs = Vec::new();

    // Parse field attributes
    for field in fields {
        if let Some(config) = parse_field_config(field) {
            field_configs.push(config);
        }
    }

    if field_configs.is_empty() {
        panic!("No valid configuration fields found");
    }

    // Generate field declarations
    let field_decls = field_configs.iter().map(|config| {
        let field_name = &config.field_name;
        let field_type = &config.field_type;
        quote! {
            #field_name: ::std::sync::Arc<::tokio::sync::RwLock<#field_type>>
        }
    });

    // Generate field names for constructor
    let field_names = field_configs.iter().map(|config| &config.field_name);

    // Generate field initialization
    let field_inits = field_configs.iter().map(|config| {
        let field_name = &config.field_name;
        let key = &config.key;
        let parse_fn = &config.parse_fn;

        if config.is_required {
            watcher::register_watcher(field_name, key, parse_fn)
        } else {
            let default_value = config.default_value.as_ref().unwrap();
            watcher::register_watcher_with_default(field_name, key, parse_fn, default_value)
        }
    });

    // Generate accessor methods
    let accessor_methods = field_configs.iter().map(|config| {
        let field_name = &config.field_name;
        let field_type = &config.field_type;
        let method_name = field_name.clone();

        quote! {
            pub async fn #method_name(&self) -> ::tokio::sync::RwLockReadGuard<'_, #field_type> {
                self.#field_name.read().await
            }
        }
    });

    let expanded = quote! {
        #[derive(::std::fmt::Debug, ::std::clone::Clone)]
        #vis struct #struct_name {
            #(#field_decls,)*
        }

        impl #struct_name {
            pub async fn new<Conf: ::dyn_cfg::facade::WatchConfCenter>(
                watch: &mut ::tokio::task::JoinSet<()>,
                cli: Conf,
            ) -> ::std::result::Result<Self, ::dyn_cfg::error::WatchInitError<Conf>> {
                use ::dyn_cfg::prelude::StreamExt;

                #(#field_inits)*

                Ok(Self {
                    #(#field_names: #field_names,)*
                })
            }

            #(#accessor_methods)*
        }
    };

    TokenStream::from(expanded)
}

fn parse_field_config(field: &Field) -> Option<FieldConfig> {
    let field_name = field.ident.as_ref()?.clone();
    let field_type = field.ty.clone();

    let mut key = None;
    let mut parse_fn = None;
    let mut default_value = None;
    let mut is_required = false;

    for attr in &field.attrs {
        if attr.path().is_ident("must_init") {
            is_required = true;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("key") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    key = Some(lit.value());
                } else if meta.path.is_ident("parse") {
                    let value = meta.value()?;
                    let parse_expr: syn::Expr = value.parse()?;
                    parse_fn = Some(parse_expr);
                }
                Ok(())
            })
            .ok()?;
        } else if attr.path().is_ident("default") {
            is_required = false;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("key") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    key = Some(lit.value());
                } else if meta.path.is_ident("parse") {
                    let value = meta.value()?;
                    let parse_expr: syn::Expr = value.parse()?;
                    parse_fn = Some(parse_expr);
                } else if meta.path.is_ident("default") {
                    let value = meta.value()?;
                    let default_expr: Expr = value.parse()?;
                    default_value = Some(default_expr);
                }
                Ok(())
            })
            .ok()?;
            if key.is_none() {
                panic!("field: {:?} key is_none", field.ident);
            }
        } else {
            panic!(
                "field: {:?} attr path is unknown: {:?}",
                field.ident,
                attr.path()
            );
        }
    }

    if field.attrs.is_empty() {
        panic!("field: {:?} attr empty", field.ident,);
    }

    if key.is_none() || parse_fn.is_none() {
        return None;
    }

    Some(FieldConfig {
        field_name,
        field_type,
        key: key.unwrap(),
        parse_fn: parse_fn.unwrap(),
        default_value,
        is_required,
    })
}
