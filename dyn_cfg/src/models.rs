use faststr::FastStr;

#[derive(Debug, Clone)]
pub enum ConfGetResult<T> {
    Exist(T),
    NotExist { key: FastStr },
    GetFail { key: FastStr, err_info: FastStr },
    ParseFail { key: FastStr, err_info: FastStr },
}

#[derive(Debug, Clone)]
pub enum ConfGetRawResult {
    Exist(FastStr),
    NotExist { key: FastStr },
    GetFail { key: FastStr, err_info: FastStr },
}

impl From<&'static str> for ConfGetRawResult {
    fn from(value: &'static str) -> Self {
        Self::Exist(FastStr::from_static_str(value))
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum ConfGetStdError {
    #[error("conf key not_exist: {key}")]
    NotExist { key: FastStr },
    #[error("get conf fail, key: {key}, err_info: {err_info}")]
    GetFail { key: FastStr, err_info: FastStr },
    #[error("parse conf fail, key: {key}, err_info: {err_info}")]
    ParseFail { key: FastStr, err_info: FastStr },
}

impl ConfGetRawResult {
    /// 按标准方式处理error，不存在也当作失败
    pub fn into_std_result(self) -> Result<FastStr, ConfGetStdError> {
        match self {
            Self::Exist(v) => Ok(v),
            Self::NotExist { key } => Err(ConfGetStdError::NotExist { key }),
            Self::GetFail { key, err_info } => Err(ConfGetStdError::GetFail { key, err_info }),
        }
    }

    /// 按标准方式处理error，不存在当作成功的一种
    pub fn into_std_result_option(self) -> Result<Option<FastStr>, ConfGetStdError> {
        match self {
            Self::Exist(v) => Ok(Some(v)),
            Self::NotExist { .. } => Ok(None),
            Self::GetFail { key, err_info } => Err(ConfGetStdError::GetFail { key, err_info }),
        }
    }

    /// 所有**失败**和**不存在**都以默认值返回
    pub fn unwrap_or(self, default_v: FastStr) -> FastStr {
        match self {
            Self::Exist(v) => v,
            Self::NotExist { .. } | Self::GetFail { .. } => default_v,
        }
    }

    /// 仅不存在时忽略错误使用默认值，失败仍返回错误
    pub fn unwrap_optional_or(self, default_v: FastStr) -> Result<FastStr, ConfGetStdError> {
        match self {
            Self::Exist(v) => Ok(v),
            Self::NotExist { .. } => Ok(default_v),
            Self::GetFail { key, err_info } => Err(ConfGetStdError::GetFail { key, err_info }),
        }
    }

    pub fn inspect_not_exist<F>(self, f: F) -> Self
    where
        F: FnOnce(FastStr),
    {
        if let Self::NotExist { key } = &self {
            f(key.clone());
        }
        self
    }

    pub fn is_not_exist(&self) -> bool {
        matches!(self, Self::NotExist { .. })
    }
}

impl<T> ConfGetResult<T> {
    /// 按标准方式处理error，不存在也当作失败
    pub fn into_std_result(self) -> Result<T, ConfGetStdError> {
        match self {
            Self::Exist(v) => Ok(v),
            Self::NotExist { key } => Err(ConfGetStdError::NotExist { key }),
            Self::GetFail { key, err_info } => Err(ConfGetStdError::GetFail { key, err_info }),
            Self::ParseFail { key, err_info } => Err(ConfGetStdError::ParseFail { key, err_info }),
        }
    }

    /// 按标准方式处理error，不存在当作成功的一种
    pub fn into_std_result_option(self) -> Result<Option<T>, ConfGetStdError> {
        match self {
            Self::Exist(v) => Ok(Some(v)),
            Self::NotExist { .. } => Ok(None),
            Self::GetFail { key, err_info } => Err(ConfGetStdError::GetFail { key, err_info }),
            Self::ParseFail { key, err_info } => Err(ConfGetStdError::ParseFail { key, err_info }),
        }
    }

    /// 所有**失败**和**不存在**都以默认值返回
    pub fn unwrap_or(self, default_v: T) -> T {
        match self {
            Self::Exist(v) => v,
            Self::NotExist { .. } | Self::GetFail { .. } | Self::ParseFail { .. } => default_v,
        }
    }

    /// 仅不存在时忽略错误使用默认值，失败仍返回错误
    pub fn unwrap_optional_or(self, default_v: T) -> Result<T, ConfGetStdError> {
        match self {
            Self::Exist(v) => Ok(v),
            Self::NotExist { .. } => Ok(default_v),
            Self::GetFail { key, err_info } => Err(ConfGetStdError::GetFail { key, err_info }),
            Self::ParseFail { key, err_info } => Err(ConfGetStdError::ParseFail { key, err_info }),
        }
    }

    pub fn inspect_not_exist<F>(self, f: F) -> Self
    where
        F: FnOnce(FastStr),
    {
        if let Self::NotExist { key } = &self {
            f(key.clone());
        }
        self
    }

    pub fn is_not_exist(&self) -> bool {
        matches!(self, Self::NotExist { .. })
    }
}
