use crate::proxy::{Proxy, ProxyTarget};
use crate::serve_static::{Multi, ServeStatic, ServeStaticConfig};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Default, StructOpt, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "serveStatic")]
    #[structopt(long = "serve-static", short = "ss")]
    pub serve_static: Option<Vec<ServeStaticConfig>>,
    #[structopt(long = "index")]
    pub index: Option<String>,
    #[structopt(long = "proxy", short = "p")]
    #[serde(default)]
    pub proxy: Vec<ProxyTarget>,
    #[structopt(parse(from_os_str))]
    #[serde(default)]
    pub trailing_paths: Vec<PathBuf>,
    #[structopt(long = "port")]
    #[serde(default = "crate::config::default_port")]
    pub port: Option<u16>,
}

pub fn default_port() -> Option<u16> {
    Some(8090)
}

pub fn get_available_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|socket_addr| socket_addr.port())
        .ok()
}

impl ServeStatic for Config {
    fn serve_static_config(&self) -> Vec<ServeStaticConfig> {
        let mut output = vec![];
        for pb in &self.trailing_paths {
            output.push(ServeStaticConfig::from_dir_only(&pb))
        }
        output.extend(self.serve_static.clone().unwrap_or_else(Vec::new));
        output
    }
    fn dir_only(&self) -> Vec<PathBuf> {
        self.serve_static_config()
            .into_iter()
            .filter_map(|ss| match ss {
                ServeStaticConfig::DirOnly(pb) => Some(pb),
                _ => None,
            })
            .collect()
    }
    fn multi_only(&self) -> Vec<Multi> {
        self.serve_static_config()
            .into_iter()
            .filter_map(|ss| match ss {
                ServeStaticConfig::Multi(multi) => Some(multi),
                _ => None,
            })
            .collect()
    }
}

impl Proxy for Config {
    fn proxies(&self) -> Vec<ProxyTarget> {
        self.proxy.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser_sync::BrowserSync;
    use crate::proxy::ProxyTarget;
    use std::str::FromStr;

    #[test]
    fn test_deserialize() -> std::io::Result<()> {
        let input = r#"
        {
            "serveStatic": [
                {
                    "routes": ["/node_modules", "react"],
                    "dir": "node_modules"
                },
                "static"
            ],
            "trailing_paths": ["."]
        }
        "#;
        let config = serde_json::from_str::<Config>(input)?;
        let ss = config.serve_static_config();
        assert_eq!(
            vec![
                ServeStaticConfig::from_dir_only("."),
                ServeStaticConfig::Multi(Multi {
                    dir: PathBuf::from("node_modules"),
                    routes: vec![String::from("/node_modules"), String::from("react")]
                }),
                ServeStaticConfig::from_dir_only("static"),
            ],
            ss
        );
        Ok(())
    }

    #[test]
    fn test_from_args() -> anyhow::Result<()> {
        let args = ". --serve-static static";
        let bs = BrowserSync::try_from_args(args.split(" "))?;
        let ss = bs.config.serve_static_config();
        assert_eq!(
            vec![
                ServeStaticConfig::from_dir_only("."),
                ServeStaticConfig::from_dir_only("static"),
            ],
            ss
        );
        Ok(())
    }

    #[test]
    fn test_from_args_with_shorthard() -> anyhow::Result<()> {
        let args = ". --serve-static node_modules:fixtures/node_modules";
        let bs = BrowserSync::try_from_args(args.split(" "))?;
        let ss = bs.config.serve_static_config();
        assert_eq!(
            vec![
                ServeStaticConfig::from_dir_only("."),
                ServeStaticConfig::Multi(Multi {
                    dir: PathBuf::from("fixtures/node_modules"),
                    routes: vec![String::from("node_modules")]
                }),
            ],
            ss
        );
        Ok(())
    }

    #[test]
    fn test_proxy_from_args() -> anyhow::Result<()> {
        let args = "--proxy http://www.example.com";
        let bs = BrowserSync::try_from_args(args.split(" "))?;
        let proxies = bs.config.proxies();
        assert_eq!(
            vec![ProxyTarget {
                target: url::Url::from_str("http://www.example.com")?,
                paths: Default::default()
            }],
            proxies
        );
        Ok(())
    }
    #[test]
    fn test_proxy_from_args_error() {
        let args = "--proxy http:/.example.com";
        let p = url::Url::parse(args);
        println!("{}", p.unwrap_err());
        // let bs = BrowserSync::try_from_args(args.split(" "));
        // assert!(bs.is_err());
    }
    #[test]
    fn test_proxy_from_json() {
        let input = r#"
        {
          "serveStatic": ["fixtures/src", "fixtures"],
          "proxies": ["/gql~https://swapi-graphql.netlify.app/.netlify/functions/index"]
        }
        "#;
        let config: Config = serde_json::from_str(input).expect("test");
        dbg!(config);
    }
}
