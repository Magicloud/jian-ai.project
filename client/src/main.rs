#![feature(backtrace)]
use futures::{stream, StreamExt, TryStreamExt};
use reqwest::{Body, Client};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "localhost:8000".to_string();
    let pic_folder = Path::new("/home/shida/pic/thumb/").to_path_buf();
    let pics = pic_folder.read_dir()?;
    let results = stream::iter(pics)
        .map_ok(|pic| upload(pic.path(), host.clone()))
        .map_err(|err| err.into())
        .try_buffer_unordered(1);
    results
        .for_each(|result| async {
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e)
                }
            }
        })
        .await;

    Ok(())
}

async fn upload(filename: PathBuf, host: String) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(filename).await?;
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);

    let client = Client::new();
    let response = client
        .post(format!(
            "http://{}/apis/new_image?camera_id=test&food_weight=1",
            host
        ))
        .body(body)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(ClientError::Upload(response.status(), response.text().await?).into())
    }
}

use reqwest::StatusCode;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
enum ClientError {
    Upload(StatusCode, String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self {
            ClientError::Upload(sc, _s) => {
                write!(f, "Upload failed as {}", sc)
            }
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        None
    }
    fn description(&self) -> &str {
        match &self {
            ClientError::Upload(_sc, s) => s,
        }
    }
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}
