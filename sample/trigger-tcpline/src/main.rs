use std::collections::HashMap;

use anyhow::Error;
use clap::{Parser};
use serde::{Deserialize, Serialize};
use spin_trigger::{cli::TriggerExecutorCommand, TriggerExecutor, TriggerAppEngine};

wit_bindgen_wasmtime::import!({paths: ["wit/tcp-line.wit"], async: *});

pub(crate) type RuntimeData = tcp_line::TcpLineData;
pub(crate) type _Store = spin_core::Store<RuntimeData>;

#[derive(clap::Args)]
struct CommandLineArgs {
    #[clap(long = "port", default_value = "127.0.0.1")]
    host: String,
}

type Command = TriggerExecutorCommand<TcpLineTrigger>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let t = Command::parse();
    t.run().await
}

struct TcpLineTrigger {
    engine: TriggerAppEngine<Self>,
    component_settings: HashMap<String, Component>,
}

// Application settings (raw serialisation format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    r#type: String,
}

// Per-component settings (raw serialisation format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerConfig {
    component: String,
    port: u16,
}

#[derive(Clone, Debug)]
struct Component {
    port: u16,
}

#[async_trait::async_trait]
impl TriggerExecutor for TcpLineTrigger {
    const TRIGGER_TYPE: &'static str = "tcpline";

    type RuntimeData = RuntimeData;

    type TriggerConfig = TriggerConfig;

    type RunConfig = CommandLineArgs;

    fn new(engine: spin_trigger::TriggerAppEngine<Self>) -> anyhow::Result<Self>  {
        let component_settings = engine
            .trigger_configs()
            .map(|(_, config)| (config.component.clone(), get_settings(config)))
            .collect();

        Ok(Self {
            engine,
            component_settings
        })
    }

    async fn run(self, config: Self::RunConfig) -> anyhow::Result<()> {
        let host = &config.host;

        // This trigger spawns threads, which Ctrl+C does not kill.  So
        // for this case we need to detect Ctrl+C and shut those threads
        // down.  For simplicity, we do this by terminating the process.
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            std::process::exit(0);
        });

        tokio_scoped::scope(|scope| {
            for (c, settings) in self.component_settings.clone() {
                let port = settings.port;

                let addr = format!("{host}:{port}");
                let listener = std::net::TcpListener::bind(&addr).unwrap();
    
                for stream in listener.incoming() {
                    let component_id = c.clone();
                    scope.spawn(async {
                        if let Ok(stm) = stream {
                            match self.handle_stream(component_id, stm).await {
                                Ok(()) => (),
                                Err(e) => { eprintln!("{e:?}"); }
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }
}

impl TcpLineTrigger {

    async fn handle_stream(&self, component_id: String, mut stm: std::net::TcpStream) -> anyhow::Result<()> {
        use std::io::{BufRead, Write};

        let reader = std::io::BufReader::new(&stm);
        let line = reader.lines().nth(0).unwrap()?;
        let response = self.handle_line(&component_id, &line).await?;
        stm.write(response.as_bytes())?;
        Ok(())
    }

    async fn handle_line(&self, component_id: &str, line: &str) -> anyhow::Result<String> {
        // Load the guest...+
        let (instance, mut store) = self.engine.prepare_instance(&component_id).await?;
        let engine = tcp_line::TcpLine::new(&mut store, &instance, |data| data.as_mut())?;
        // ...and call the entry point
        let response = engine
            .handle_line(&mut store, line)
            .await?;
        Ok(response)

    }
}

fn get_settings(config: &TriggerConfig) -> Component {
    Component {
        port: config.port
    }
}
