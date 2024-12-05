//! TODO: implement a file download manager using async/await
//!
//! It should go through a list of links and download them all to a specified directory as fast
//! as possible, while periodically displaying progress and download speed.
//! Everything should happen on a single thread, it is not needed to create more threads manually.
//!
//! There are two test files with URLs that you can use for testing:
//! - `links-small.txt`
//! - `links-medium.txt`
//!
//! Start with a simple solution, and then incrementally make it better:
//! 1) Download links one-by-one.
//! For each link, first download the whole file to memory, then write it to disk.
//! Print progress in-between file downloads.
//! Use [`reqwest::Client`] for downloading the links. A simple GET request should be enough.
//!
//! 2) Download links one-by-one.
//! For each link, overlap the network download with writing the file to disk.
//! Create two futures:
//! - One will download the file using HTTP chunking (see [`reqwest::Response::chunk`]
//!   or [`reqwest::Response::bytes_stream`])
//! - The second will concurrently write the chunks to the destination file.
//!
//! Connect the two futures using a Tokio MPSC channel.
//!
//! Wait until both futures are completed. Remember, they should run concurrently!
//! You can use e.g. [`tokio::join`] or [`futures::future::join`] for this.
//!
//! 2a) Add periodic output (every 500ms) that will show the download progress (in %) and the
//!   download speed (in MiB/s, see [`humansize::format_size`] and [`humansize::BINARY`]).
//!   You can use [`tokio::select`] to overlap the periodic print with the network download.
//!   When using futures inside [`tokio::select`] branches, you might need to pin them using
//!   [`Box::pin`] (on the heap) or [`std::pin::pin`] (on the stack).
//!
//! 2b) Add disk I/O speed to the periodic output. This means that you will have to perform the
//!   output outside the two (network and disk) futures, and share state between them.
//!
//! 3) Download the files concurrently. You can simply spawn each download using
//! [`tokio::task::spawn_local`] and download everything at once.
//!
//! 4) Download the files concurrently, but only N files at once, to avoid overloading the
//! network interface/disk.
//! You can use e.g. [`tokio::task::JoinSet`] to execute N futures concurrently, periodically
//! read results of resolved futures, and add new futures.

use futures::StreamExt;
use humansize::BINARY;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::task::{JoinSet, LocalSet};
use tokio::time::Instant;
use url::Url;

#[derive(Debug)]
struct DownloadEntry {
    url: Url,
    file_name: String,
}

fn main() -> anyhow::Result<()> {
    let links: Vec<DownloadEntry> = std::fs::read_to_string("links-small.txt")?
        .lines()
        .map(|s| {
            let url = Url::parse(s)?;
            let file_name = url.path_segments().unwrap().last().unwrap().to_string();
            Ok(DownloadEntry { url, file_name })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let dest = PathBuf::from("downloads");
    if dest.is_dir() {
        std::fs::remove_dir_all(&dest)?;
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let start = Instant::now();
    runtime.block_on(async move {
        let localset = LocalSet::new();
        localset.run_until(download_files(links, dest)).await
    })?;
    println!("Duration: {:.2}s", start.elapsed().as_secs_f64());

    Ok(())
}

/// TODO: implement file download
async fn download_files(links: Vec<DownloadEntry>, dest: PathBuf) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&dest).await?;
    for link in links {
        println!("Starting download: {}", link.url);

        let (tx, mut rx) = tokio::sync::mpsc::channel(256);

        let network_downloader = async move {
            // Download the file content
            let client = reqwest::Client::new();
            let response = client.get(link.url.clone()).send().await.unwrap().error_for_status().unwrap();
            let mut stream = response.bytes_stream();

            while let Some(Ok(chunk)) = stream.next().await {
                tx.send(chunk).await.unwrap();
            }
        };

        let dest = dest.join(&link.file_name);
        let disk_writer = async move {
            // Write to the destination file
            let mut file = tokio::fs::File::create(&dest).await.unwrap();

            while let Some(chunk) = rx.recv().await {
                file.write_all(&chunk).await.unwrap();
            }
        };
        let printer = async move {
            
        }
        tokio::join!(
            network_downloader,
            disk_writer,
        );
    }

    Ok(())
}

