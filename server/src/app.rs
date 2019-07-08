use std::{
    collections::BTreeMap,
    fs::File,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, RwLock},
};

use clap::{App as ClapApp, Arg as ClapArg};

use serde::Deserialize;

use crate::{
    database::MemoryDb,
    plugin::{load_plugins, Plugin},
    task::TaskState,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub(crate) struct AppConfig {
    pub plugins_path: String,
    pub bind_address: SocketAddr,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            plugins_path: "./".into(),
            bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000),
        }
    }
}

fn load_config_from_file(path: String) -> Result<AppConfig, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    serde_json::from_reader::<_, AppConfig>(file).map_err(|e| e.to_string())
}

impl AppConfig {
    pub(crate) fn from_clap() -> Self {
        let cli = ClapApp::new("Task processing server")
            .version("1.0")
            .author("Andrey \"Flowneee\" Kononov. <flowneee3@gmail.com>")
            .about("Uploads image and generates preview")
            .arg(
                ClapArg::with_name("bind-address")
                    .short("a")
                    .long("bind-address")
                    .value_name("ADDRESS")
                    .help("Server bind address")
                    .takes_value(true)
                    .default_value("127.0.0.1:8000")
                    .validator(|x| {
                        x.parse::<SocketAddr>()
                            .map(|_| ())
                            .map_err(|x| x.to_string())
                    }),
            )
            .arg(
                ClapArg::with_name("plugins-path")
                    .short("p")
                    .long("plugins-path")
                    .value_name("PATH")
                    .help("Location of plugins")
                    .takes_value(true)
                    .default_value("./"),
            )
            .arg(
                ClapArg::with_name("config-path")
                    .short("c")
                    .long("config-path")
                    .value_name("PATH")
                    .help("Path to configuration file (in JSON format)")
                    .takes_value(true),
            )
            .get_matches();

        let mut base_config = match cli.value_of("config-path") {
            Some(x) => load_config_from_file(x.into()).unwrap(),
            None => AppConfig::default(),
        };
        if let Some(x) = cli.value_of("bind-address") {
            base_config.bind_address = x.parse().unwrap_or_else(|_| {
                unimplemented!("'bind-address' should be validated inside clap")
            });
        };
        if let Some(x) = cli.value_of("plugins-path") {
            base_config.plugins_path = x.into()
        };

        base_config
    }
}

#[derive(Clone)]
pub(crate) struct App {
    pub plugins: Arc<RwLock<BTreeMap<String, Plugin>>>,
    pub database: Arc<MemoryDb<TaskState>>,
    pub stop_tasks_flag: Arc<RwLock<()>>,
    pub config: AppConfig,
}

impl App {
    pub(crate) fn from_config(config: AppConfig) -> Result<Self, ()> {
        let database = Arc::new(MemoryDb::new());
        let plugins = Arc::new(RwLock::new(
            load_plugins(config.plugins_path.clone()).unwrap(),
        ));
        Ok(Self {
            plugins,
            database,
            stop_tasks_flag: Arc::new(RwLock::new(())),
            config,
        })
    }
}
