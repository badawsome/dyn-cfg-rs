use faststr::FastStr;

use crate::models::{ConfGetRawResult, ConfGetResult};

macro_rules! get_with_handler {
    ($self: ident, $key: ident, $fn: expr) => {
        async move {
            let raw = match $self.get_raw($key.clone()).await {
                ConfGetRawResult::Exist(res) => res,
                ConfGetRawResult::NotExist { key } => {
                    return ConfGetResult::NotExist { key };
                }
                ConfGetRawResult::GetFail { key, err_info } => {
                    return ConfGetResult::GetFail { key, err_info };
                }
            };
            match $fn(raw.as_str()) {
                Ok(res) => ConfGetResult::Exist(res),
                Err(e) => ConfGetResult::ParseFail {
                    key: $key.clone(),
                    err_info: FastStr::new(format!("{}", e)),
                },
            }
        }
    };
}

pub trait ConfCenter: ConfCenterBasic {
    fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        key: FastStr,
    ) -> impl std::future::Future<Output = ConfGetResult<T>> + Send {
        get_with_handler!(self, key, serde_json::from_str)
    }

    fn get_bool(
        &self,
        key: FastStr,
    ) -> impl std::future::Future<Output = ConfGetResult<bool>> + Send {
        get_with_handler!(self, key, |x: &str| { x.parse::<bool>() })
    }

    fn get_from_str<E: std::error::Error, T: std::str::FromStr<Err = E>>(
        &self,
        key: FastStr,
    ) -> impl std::future::Future<Output = ConfGetResult<T>> + Send {
        get_with_handler!(self, key, |x: &str| { x.parse::<T>() })
    }

    /// 提供 systemd 定义的 duration 格式解析
    /// 格式详细信息见 https://www.freedesktop.org/software/systemd/man/latest/systemd.time.html#Parsing%20Time%20Spans
    /// ```rust
    /// fn parse() {
    ///     use parse_duration::parse;
    ///     let duration = parse("1h30m").unwrap();
    ///     assert_eq!(duration, std::time::Duration::from_secs(5400));
    /// }
    /// ```
    fn get_systemd_duration(
        &self,
        key: FastStr,
    ) -> impl std::future::Future<Output = ConfGetResult<std::time::Duration>> + Send {
        get_with_handler!(self, key, parse_duration::parse)
    }
}

pub trait ConfCenterBasic: Send + Sync + std::fmt::Display {
    fn get_raw(&self, key: FastStr) -> impl std::future::Future<Output = ConfGetRawResult> + Send;
}

pub trait WatchConfCenter: Clone + ConfCenterBasic + 'static {
    fn watch_raw(
        &self,
        key: FastStr,
    ) -> impl futures::stream::Stream<Item = ConfGetRawResult> + Unpin + Send + Sync + 'static;
}
