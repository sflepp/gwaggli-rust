use cli::commands::run;

mod audio;
mod cli;
mod environment;
mod event_system;
mod task;
mod transcription;
mod util;

#[tokio::main]
async fn main() {
    println!("{}", run().await.unwrap());
}
