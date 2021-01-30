use clap::Clap;
use futures::future::join_all;
use rayon::prelude::*;
use std::fs::File;
use std::io::copy;
use std::io::{self, BufRead};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let urls: Vec<String> = match read_lines(opts.input_file) {
        Ok(lines) => lines.filter_map(|l| match l {
            Err(err) => {
                panic!(err)
            }
            Ok(line) => Some(line),
        }),
        Err(err) => {
            panic!(err)
        }
    }
    .collect();
    let output_path = Path::new(&opts.output_folder);

    let urls_to_download: Vec<_> = urls
        .par_iter()
        .filter_map(|url| {
            let file_name = get_url_file_name(url);
            if !output_path.join(&file_name).exists() {
                Some((url, file_name))
            } else {
                None
            }
        })
        .map(|(url, file_name)| download_url(output_path, (url, file_name)))
        .collect();

    join_all(urls_to_download).await;

    Ok(())
}

async fn download_url(output_path: &Path, (url, file_name): (&String, String)) {
    println!("Downloading :  '{}' - '{}'", url, file_name);
    let resp = reqwest::get(url).await;
    if let Ok(resp) = resp {
        let mut dest = {
            let fname = resp
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or(&file_name);
            let fname = output_path.join(fname);
            File::create(fname).unwrap()
        };
        let bytes = resp.bytes().await.unwrap();
        let bufsved = bytes.to_vec();
        let mut bufs = bufsved.as_slice();
        copy(&mut bufs, &mut dest).unwrap();
        println!("Downloaded :  '{}' - '{}'", url, file_name);
    }
}

pub fn get_url_file_name(url: &String) -> String {
    match url.split("/").last() {
        Some(name) => name.to_string(),
        None => panic!("Could not find filename for url: {}", url),
    }
}

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Philip Kristoffersen <philipkristoffersen@gmail.com>"
)]
struct Opts {
    #[clap(about = "Text file with one url pr line that should be downloaded")]
    input_file: String,

    #[clap(
        short,
        long,
        default_value = ".",
        about = "the output folder to download the files to"
    )]
    output_folder: String,
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
