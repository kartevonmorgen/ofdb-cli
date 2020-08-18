use std::{fs::File, io, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ofdb", about = "CLI for OpenFairDB", author)]
enum Opt {
    #[structopt(about = "Import new entries")]
    Import {
        #[structopt(parse(from_os_str), help = "JSON file")]
        file: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    match opt {
        Opt::Import { file } => import(file),
    }
}

fn import(path: PathBuf) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let entries: Vec<ofdb_boundary::NewPlace> = serde_json::from_reader(reader)?;
    log::debug!("Read {} entries from JSON file", entries.len());
    println!("The import feature is not implemented yet"); // TODO
    Ok(())
}
