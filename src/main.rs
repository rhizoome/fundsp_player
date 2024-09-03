#[cfg(debug_assertions)]
use assert_no_alloc::AllocDisabler;
use clap::{Parser, Subcommand};
use runner::{dummy, live};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[derive(Parser, Debug)]
#[command(version, about = "FunDSP player")]
struct Args {
    #[arg(help = "DSP build to play")]
    build: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Play to audio device")]
    Live {
        #[arg(short, long, help = "Audio device - if not found a list is displayed")]
        device: Option<String>,
    },
    #[command(about = "Play to 'devnull'")]
    Dummy {
        #[arg(short, long, default_value_t = 10, help = "Seconds to generate")]
        seconds: u32,
    },
    #[command(about = "Play to wave file")]
    File {
        #[arg(short, long, default_value_t = 10, help = "Seconds to generate")]
        seconds: u32,
        #[arg(
            short,
            long,
            default_value_t ={
                #[allow(unused_parens)]
                ("output.wav".into())
            },
            help = "Output file"
        )]
        filename: String,
    },
}

fn main() {
    let args = Args::parse();
    match &args.command {
        Commands::Live { device } => live(device.as_deref(), &args.build),
        Commands::Dummy { seconds } => dummy(*seconds, &args.build),
        Commands::File { .. } => todo!(),
    }
}

mod build;
mod runner;
