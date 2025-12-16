pub mod prelude {
    pub use crate::{error::*, facade::*, mock::*, models::*};
    pub use anyhow;
    pub use dyn_cfg_macros::dynamic_config;
    pub use faststr::FastStr;
    pub use futures::StreamExt;
    pub use tokio::{self, task::JoinSet};
    pub use tracing;
}

pub mod error;
pub mod facade;
pub mod mock;
pub mod models;
