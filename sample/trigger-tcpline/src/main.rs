use std::{collections::HashMap, sync::Arc};

use anyhow::{Error, anyhow};
use clap::{Parser};
use serde::{Deserialize, Serialize};
use spin_trigger::{cli::TriggerExecutorCommand, TriggerExecutor, TriggerAppEngine};

wit_bindgen_wasmtime::import!({paths: ["wit/tcp-line.wit"], async: *});

pub(crate) type RuntimeData = tcp_line::TcpLineData;
pub(crate) type _Store = spin_core::Store<RuntimeData>;

// The options we want to surface via the `spin up` command line.
// No, I couldn't think of a very realistic one!
#[derive(clap::Args)]
struct CommandLineArgs {
    #[clap(long = "host", default_value = "127.0.0.1")]
    host: String,
}

// `TriggerExecutorCommand` is the Spin type that handles
// program, application, and trigger load. Parameterise it
// with the trigger type.
type Command = TriggerExecutorCommand<TcpLineTrigger>;

// The entry point for the trigger plugin. Just a normal
// Rust program...
#[tokio::main]
async fn main() -> Result<(), Error> {
    // ...that immediately hands off to TriggerExecutorCommand
    // to do all the heavy lifting.
    let t = Command::parse();
    t.run().await
}

// The trigger type is defined by the trigger plugin author.
// Spin only requires that it implements TriggerExecutor. In
// practice it almost always has an `engine` and some kind of
// collection of component settings. App level settings might
// go here or be folded into the component settings. All of
// this can be captured in the `new` method.
struct TcpLineTrigger {
    engine: TriggerAppEngine<Self>,
    component_settings: HashMap<String, Component>,
}

// Application settings (raw serialisation format)
// These correspond to what you see in `trigger = { ... }`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    r#type: String,
}

// Per-component settings (raw serialisation format)
// These correspond to what you see under `[component.trigger]`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerConfig {
    component: String,
    port: u16,
}

// Often it's useful to have an internal representation of
// the component settings, for example if things need parsing, or
// options need defaulting. It's overkill here but showing it for
// the pattern. See the SQS trigger for a more realistic example.
#[derive(Clone, Debug)]
struct Component {
    port: u16,
}

#[async_trait::async_trait]
impl TriggerExecutor for TcpLineTrigger {
    const TRIGGER_TYPE: &'static str = "tcpline";

    type RuntimeData = RuntimeData;

    // What per-component settings look like. Accessed through
    // in `new` via `engine.trigger_configs`.
    type TriggerConfig = TriggerConfig;

    // Tells TriggerExecutorCommand extra fields for the command line.
    // Accessed in `run` via the `config` parameter.
    type RunConfig = CommandLineArgs;

    fn new(engine: spin_trigger::TriggerAppEngine<Self>) -> anyhow::Result<Self>  {
        let component_settings = engine
            .trigger_configs()
            .map(|(_, config)| (config.component.clone(), get_settings(config)))
            .collect();

        // We don't have any app level settings, but if we did, we
        // could get them via `engine.app().require_metadata::<TriggerMetadata>("trigger")`.

        Ok(Self {
            engine,
            component_settings
        })
    }

    async fn run(self, config: Self::RunConfig) -> anyhow::Result<()> {
        let host = &config.host;
        let engine = Arc::new(self.engine);

        // This trigger spawns threads, which Ctrl+C does not kill.  So
        // for this case we need to detect Ctrl+C and shut those threads
        // down.  For simplicity, we do this by terminating the process.
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            std::process::exit(0);
        });

        let loops = self.component_settings.iter().map(|(c, settings)| {
            let port = settings.port;
            let addr = format!("{host}:{port}");
            // Move things out to a function that does not borrow `self`.
            // This makes lifetimes much easier!
            tokio::task::spawn(Self::run_listen_loop(engine.clone(), c.clone(), addr))
        });

        let (fin, _, rest) = futures::future::select_all(loops).await;
        drop(rest);

        fin.map_err(|e| anyhow!(e))
    }
}

impl TcpLineTrigger {

    // BEWARE! The error handling in the loop and the functions it calls
    // is horribly oversimplified. An error processing a request should
    // *not* exit the loop, because that ends the program!

    async fn run_listen_loop(engine: Arc<TriggerAppEngine<Self>>, component_id: String, addr: String) {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        println!("{component_id} listening on {addr}");
        loop {
            if let Ok((stm, _)) = listener.accept().await {
                let stm = stm.into_std().unwrap();
                stm.set_nonblocking(false).unwrap();
                match Self::handle_stream(engine.clone(), component_id.clone(), stm).await {
                    Ok(()) => (),
                    Err(e) => { eprintln!("handle_stream failed {e:?}"); }
                }
            }
        }
    }

    async fn handle_stream(engine: Arc<TriggerAppEngine<Self>>, component_id: String, mut stm: std::net::TcpStream) -> anyhow::Result<()> {
        use std::io::{BufRead, Write};

        let reader = std::io::BufReader::new(&stm);
        let line = reader.lines().nth(0).unwrap()?;
        let response = Self::handle_line(engine, &component_id, &line).await?;
        stm.write(response.as_bytes())?;
        Ok(())
    }

    async fn handle_line(engine: Arc<TriggerAppEngine<Self>>, component_id: &str, line: &str) -> anyhow::Result<String> {
        // Load the guest...
        let (instance, mut store) = engine.prepare_instance(&component_id).await?;
        let engine = tcp_line::TcpLine::new(&mut store, &instance, |data| data.as_mut())?;
        // ...and call the entry point
        let response = engine
            .handle_line(&mut store, line)
            .await?;
        Ok(response)

    }
}

// Again, this is overkill here, but useful in more complex
// cases.
fn get_settings(config: &TriggerConfig) -> Component {
    Component {
        port: config.port
    }
}
