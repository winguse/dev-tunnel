mod client;
mod server;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author = "Yingyu Cheng <1443504+winguse@users.noreply.github.com>",
    version = "0.0.1",
    about = "CLI for Dev Tunnel"
)]
struct Args {
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Parser, Debug)]
enum Mode {
    Server(ServerArgs),
    Client(ClientArgs),
}

#[derive(Parser, Debug)]
struct ServerArgs {
    #[clap(long, value_name = "IP:PORT", default_value = "127.0.0.1:3000")]
    listen: String,

    #[clap(long, value_name = "INTERFACE", default_value = "eth0")]
    interface: String,

    #[clap(long, value_name = "PORTS", value_delimiter = ' ', num_args = 0..)]
    dmz_ports: Vec<String>,
}

#[derive(Parser, Debug)]
struct ClientArgs {
    #[clap(long, value_name = "URL", default_value = "ws://127.0.0.1:3000")]
    connect: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.mode {
        Mode::Server(server_args) => {
            server::run_server(
                &server_args.listen,
                &server_args.interface,
                &server_args.dmz_ports,
            )
            .await;
        }
        Mode::Client(client_args) => {
            client::run_client(&client_args.connect).await;
        }
    }
}
