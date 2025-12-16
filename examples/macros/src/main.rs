use dyn_cfg::prelude::*;
use faststr::FastStr;
use std::collections::HashMap;

#[dynamic_config]
struct Config {
    #[must_init(key = "log.level", parse = (|x: &str| x.parse::<u32>()))]
    log_level: u32,
    #[default(key = "mp", default = HashMap::new(), parse = serde_json::from_str)]
    mp: HashMap<FastStr, FastStr>,
}

#[tokio::main]
async fn main() {
    let mut watch = tokio::task::JoinSet::new();
    let cli = MockConfCenterBasic::default();
    cli.insert_and_wait("log.level", "10".into()).await;
    match Config::new(&mut watch, cli.clone()).await {
        Ok(x) => {
            let _ = dbg!(Config::keys());
            let _ = dbg!(x.log_level().await);
            let _ = dbg!(x.mp().await);
            cli.insert_and_wait("mp", r#"{"a": "b"}"#.into()).await;
            let _ = dbg!(x.mp().await);
        }
        Err(e) => eprintln!("init config fail: {}", e),
    }
}
