use cli::cli::run;

mod audio;
mod transcription;
mod cli;
mod environment;

#[tokio::main]
async fn main() {
    println!("{}", run().await.unwrap());
}
