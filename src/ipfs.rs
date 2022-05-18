use futures::{stream, StreamExt};
use reqwest::{
    multipart,
    multipart::{Form, Part},
    Body, Client,
};
use std::sync::{Arc, Condvar, RwLock};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

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

        //for file_path in file_paths {
        //    let file = File::open(file_path).await?;
        //    form.part("file", file_to_body(file));
        //}
        //
        let bodies = stream::iter(file_paths)
            .map(|path| self.process_path(path.to_owned()))
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

        loop {
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

            return Ok(response);
        }
    }

    //       let new_form: Arc<RwLock<Form>> = bodies.fold(form, move |form_move, body| {
    //           if let Ok(form_locked) = form.write() {
    //               form_locked.part("file", body);
    //           }

    //           form
    //       });
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
