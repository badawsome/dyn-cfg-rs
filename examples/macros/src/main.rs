use dyn_cfg::prelude::*;
use faststr::FastStr;
use std::collections::HashMap;

#[dynamic_config]
struct Config {
    #[must_init(key = "log.level", parse = (|x: FastStr| x.parse::<u32>()))]
    log_level: u32,
    #[default(key = "f", default = HashMap::new(), parse = (|x: FastStr| serde_json::from_str(x.as_str())))]
    f: HashMap<FastStr, FastStr>,
}

#[tokio::main]
async fn main() {
    let mut watch = JoinSet::new();
    let cli = MockConfCenterBasic::default();
    cli.insert("log.level", "10".into());
    match Config::new(&mut watch, cli).await {
        Ok(x) => {
            let _ = dbg!(x.log_level().await);
        }
        Err(e) => eprintln!("init config fail: {}", e),
    }
}
