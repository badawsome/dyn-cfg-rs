use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident};

pub fn register_watcher(field_name: &Ident, key: &String, parse_fn: &Expr) -> TokenStream {
    let p = watch_raw(parse_fn);
    quote! {
        let #field_name = {
            use ::dyn_cfg::models::ConfGetRawResult;
            use ::dyn_cfg::error::WatchInitError;
            use ::dyn_cfg::prelude::faststr::FastStr;
            let key = FastStr::from_static_str(#key);
            let val_p = ::std::sync::Arc::new(tokio::sync::RwLock::new(
                match cli.get_raw(key.clone()).await {
                    ConfGetRawResult::Exist(s) => #parse_fn(s.as_str()).map_err(|e| WatchInitError::CannotParse{
                        cli:cli.clone(), key:key.clone(), err_info: FastStr::new(format!("{}", e)),
                    })?,
                    ConfGetRawResult::NotExist { .. } => {
                        return Err(WatchInitError::RequiredKeyNotExist{
                            cli:cli.clone(), key:key.clone(),
                        });
                    },
                    ConfGetRawResult::GetFail { err_info, .. } => {
                        return Err(WatchInitError::GetFail{
                            cli:cli.clone(), key:key.clone(), err_info,
                        });
                    },
                },
            ));
            #p;
            val_p
        };
    }
}

pub fn register_watcher_with_default(
    field_name: &Ident,
    key: &String,
    parse_fn: &Expr,
    default_v: &Expr,
) -> TokenStream {
    let p = watch_raw(parse_fn);
    quote! {
        let #field_name = {
            use ::dyn_cfg::models::ConfGetRawResult;
            use ::dyn_cfg::error::WatchInitError;
            use ::dyn_cfg::prelude::faststr::FastStr;
            let key = FastStr::from_static_str(#key);
            let val_p = ::std::sync::Arc::new(tokio::sync::RwLock::new(
                match cli.get_raw(key.clone()).await {
                    ConfGetRawResult::Exist(s) => #parse_fn(s.as_str()).map_err(|e| WatchInitError::CannotParse{
                        cli:cli.clone(), key:key.clone(), err_info: FastStr::new(format!("{}", e)),
                    })?,
                    ConfGetRawResult::NotExist { .. } => #default_v,
                    ConfGetRawResult::GetFail { err_info, .. } => {
                        return Err(WatchInitError::GetFail{
                            cli:cli.clone(), key:key.clone(), err_info,
                        });
                    },
                },
            ));
            #p;
            val_p
        };
    }
}

pub fn watch_raw(parse_fn: &Expr) -> TokenStream {
    quote! {
        let val_p_c = val_p.clone();
        let cli_x = cli.clone();
        let mut x = cli_x.watch_raw(key.clone());
        watch.spawn(async move {
            use ::dyn_cfg::models::ConfGetRawResult;
            use ::dyn_cfg::prelude::tracing;
            while let Some(i) = x.next().await {
                match i {
                    ConfGetRawResult::Exist(s) => match #parse_fn(s.as_str()) {
                        Ok(v) => *val_p_c.write().await = v,
                        Err(e) => {
                            tracing::event!(
                                tracing::Level::ERROR,
                                err = %e,
                                cli = %cli_x,
                                key = %&key,
                                val_raw = %s,
                                "parse fail, will use cache val"
                            );
                        }
                    },
                    ConfGetRawResult::NotExist { .. } => {
                        tracing::event!(
                            tracing::Level::ERROR,
                            cli = %cli_x,
                            key = %&key,
                            "not allow to delete key, will use cache val"
                        );
                    }
                    ConfGetRawResult::GetFail { err_info, .. } => {
                        tracing::event!(
                            tracing::Level::ERROR,
                            cli = %cli_x,
                            key = %&key,
                            err_info = %err_info,
                            "refresh value fail, will use cache val"
                        );
                    }
                }
            }
        })
    }
}
