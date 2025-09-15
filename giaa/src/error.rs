use thiserror::Error;

#[derive(Error, Debug)]
pub enum GiaaError {
    #[error("右键退出程序")]
    RightClickExit,
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}
