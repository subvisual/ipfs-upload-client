use futures::{stream, StreamExt};
use reqwest::{multipart, multipart::Part, Body, Client};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[allow(clippy::upper_case_acronyms)]
pub struct IPFS {
    client: Client,
    url: String,
    id: String,
    secret: String,
}

impl IPFS {
    pub fn new(url: String, id: String, secret: String) -> Self {
        IPFS {
            client: Client::new(),
            url,
            id,
            secret,
        }
    }

    pub async fn add_file(&self, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{url}/api/v0/add", url = self.url);
        let token = format!("{}:{}", self.id, self.secret);

        let file = File::open(file_path).await?;

        let form = multipart::Form::new().part("file", file_to_body(file));

        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .multipart(form)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
    pub async fn add_directory(
        &self,
        file_paths: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut form = multipart::Form::new();

        let bodies = stream::iter(file_paths)
            .map(|path| self.process_path(path))
            .buffer_unordered(100)
            .collect::<Vec<Result<Part, Box<dyn std::error::Error>>>>()
            .await;

        for b in bodies {
            match b {
                Ok(body) => {
                    form = form.part("file", body);
                }
                Err(e) => eprintln!("{}", e),
            }
        }

        let url = format!("{url}/api/v0/add?wrap-with-directory=true", url = self.url);
        let token = format!("{}:{}", self.id, self.secret);
        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .multipart(form)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }

    async fn process_path(&self, path: String) -> Result<Part, Box<dyn std::error::Error>> {
        let future_file = File::open(path);
        let file: File = future_file.await?;

        let body = file_to_body(file);
        Ok(body)
    }
}

fn file_to_body(file: File) -> Part {
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);

    Part::stream(body)
}
