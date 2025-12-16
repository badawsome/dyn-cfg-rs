use faststr::FastStr;

#[derive(Debug, thiserror::Error, Clone)]
pub enum WatchInitError<C: crate::facade::WatchConfCenter> {
    #[error("[dyn_cfg] key is required but not exist, confcenter: {}, key: {}", .cli, .key)]
    RequiredKeyNotExist { cli: C, key: FastStr },

    #[error("[dyn_cfg] value parse fail, confcenter: {}, key: {}, err_info: {}", .cli, .key, .err_info)]
    CannotParse {
        cli: C,
        key: FastStr,
        err_info: FastStr,
    },

    #[error("[dyn_cfg] get value fail, confcenter: {}, key: {}, err_info: {}", .cli, .key, .err_info)]
    GetFail {
        cli: C,
        key: FastStr,
        err_info: FastStr,
    },
}
