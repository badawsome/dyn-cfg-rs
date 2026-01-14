use dyn_cfg::prelude::*;
use faststr::FastStr;
use std::collections::HashMap;

#[dynamic_config]
struct Config {
    #[must_init(key = "log.level", parse = (|x: &str| x.parse::<u32>()))]
    log_level: u32,
    #[default(key = "mp", default = HashMap::new(), parse = serde_json::from_str)]
    mp: HashMap<FastStr, FastStr>,
    #[default(key = "mapping.with_default", default = MappingConfigWithDefault::default(false), parse = serde_json::from_str)]
    mapping_with_default: MappingConfigWithDefault<u32, bool>,
    #[default(key = "cli_test", default = "default".to_string(), parse_with_cli = parse_with_cli_fn)]
    cli_test: String,
}

async fn parse_with_cli_fn(cli: &impl WatchConfCenter, val: &str) -> anyhow::Result<String> {
    let res = cli.get_raw("mp".into()).await.into_std_result()?;
    Ok(format!("{}_{}", res, val))
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
            cli.insert_and_wait(
                "mapping.with_default",
                r#"{"mapping":{"1": true, "2": false}, "default": true}"#.into(),
            )
            .await;
            let _ = dbg!(x.mapping_with_default().await);
            cli.insert_and_wait("cli_test", "test_val".into()).await;
            let _ = dbg!(x.cli_test().await);
        }
        Err(e) => eprintln!("init config fail: {}", e),
    }
}
