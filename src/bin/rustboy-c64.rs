use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Kernal ROM
    #[clap(short, long, value_parser)]
    kernal: Option<String>,
}

fn main() -> Result<(), ()> {
    let _ = Args::parse();
    return Ok(());
}
