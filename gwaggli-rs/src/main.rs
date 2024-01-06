use cli::cli::run;

mod audio;
mod cli;
mod environment;
mod transcription;

#[tokio::main]
async fn main() {
    println!("{}", run().await.unwrap());
}
