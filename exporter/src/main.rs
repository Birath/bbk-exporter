use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 10032)]
    port: u16,

    #[arg(short, long, default_value = "bbk")]
    bbk: std::path::PathBuf,

    #[arg(last = true)]
    bbk_args: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let _ = bbk_exporter::run_exporter(args.port, args.bbk, args.bbk_args).await;
}
