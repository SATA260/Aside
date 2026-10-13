use thiserror::Error;

/// 所有命令共用的错误。`Invalid` 的文字会原样显示给用户，所以要写成能看懂的中文。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Invalid(String),
    #[error("数据库出错：{0}")]
    Db(#[from] rusqlite::Error),
    #[error("读写文件出错：{0}")]
    Io(#[from] std::io::Error),
    #[error("数据格式不对：{0}")]
    Json(#[from] serde_json::Error),
}

pub type AppResult<T> = Result<T, AppError>;

pub fn invalid<T>(msg: impl Into<String>) -> AppResult<T> {
    Err(AppError::Invalid(msg.into()))
}
