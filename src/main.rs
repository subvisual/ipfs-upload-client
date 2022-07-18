mod ipfs;

use crate::ipfs::IPFS;
use clap::Parser;
use std::fs::metadata;
use std::fs::read_dir;

const INFURA_API: &str = "https://ipfs.infura.io:5001";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    id: String,
    #[clap(short, long)]
    secret: String,
    #[clap(short, long, default_value_t = String::from(INFURA_API))]
    url: String,
    #[clap(short, long, default_missing_value = "true")]
    pin: String,
    #[clap(help = "The file path or directory to upload")]
    path: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let id = args.id;
    let secret = args.secret;
    let url = args.url;
    let path = args.path;

    let is_file: bool = metadata(path.clone()).expect("not a valid path").is_file();

    if is_file {
        let api = IPFS::new(url, id, secret);
        let out = api.add_file(&path).await;
        println!("{:?}", out);
    } else {
        let paths = read_dir(&path)
            .expect("not a valid dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let api = IPFS::new(url, id, secret);
        let out = api.add_directory(paths).await;
        println!("{:?}", out);
    }
}
