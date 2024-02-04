use cli::commands::run;

mod audio;
mod cli;
mod environment;
mod transcription;
mod utils;

#[tokio::main]
async fn main() {
    println!("{}", run().await.unwrap());
}
