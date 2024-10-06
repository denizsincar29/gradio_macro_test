// This is a simple example of how to use the generated API
// The api is generated in compile time, iliminating the need to learn the API and the need to write the boilerplate code
// One simple macro fetches the api using the gradio crate and generates the api for you.
// notice 1: You need to be online to build the project for the first time, because the macro fetches the api from the server.
// notice 2: for some reason, the macro relies on serde and serde_json crate, so cargo add them to the project where you use the macro.
// we highly recommend to use an IDE that supports rust analyzer to determin the api struct methods since the api is generated in compile time.

use std::fs;

use gradio::{structs::QueueDataMessage, PredictionOutput, PredictionStream};
use gradio_macro::gradio_api;


// The macro generates the api struct for you, so you don't need to write the struct yourself.
#[gradio_api(url = "hf-audio/whisper-large-v3-turbo", option = "async")]
pub struct WhisperLarge;

pub async fn show_progress(stream: &mut PredictionStream) -> Option<Vec<PredictionOutput>> {
    while let Some(message) = stream.next().await {
        if let Err(val) = message {
            eprintln!("Error: {:?}", val);
            continue;
            // return None;  // skip the error and continue
        }
        match message.unwrap() {
            QueueDataMessage::Open => println!("Task started"),
            QueueDataMessage::Progress { event_id: _, eta, progress_data } => {
                println!("Processing: (ETA: {:?})", eta);
                if let Some(progress_data) = progress_data {
                    let progress_data = &progress_data[0];
                    println!("Progress: {:?} / {:?} {:?}", progress_data.index+1, progress_data.length.unwrap_or(0), progress_data.unit);
                }
            },
            QueueDataMessage::ProcessCompleted { event_id: _, output, success } => {
                if !success {
                    eprintln!("Failed");
                    return None;
                }
                println!("Completed!");
                return Some(output.try_into().unwrap());
            },
            QueueDataMessage::Heartbeat => {},  // my heart is beating, don't worry
            QueueDataMessage::Estimation { event_id: _, rank, queue_size, rank_eta } => {
                println!("In queue: {}/{} (ETA: {:?})", rank+1, queue_size, rank_eta);
            },
            QueueDataMessage::Log { event_id } => {
                println!("Log: {}", event_id.unwrap_or("_".to_string()));
            },
            QueueDataMessage::ProcessStarts { event_id: _, eta, progress_data } => {
                println!("Processing: (ETA: {:?})", eta);
                if let Some(progress_data) = progress_data {
                    let progress_data = &progress_data[0];
                    println!("Progress: {:?} / {:?} {:?}", progress_data.index+1, progress_data.length.unwrap_or(0), progress_data.unit);
                }
            },
            QueueDataMessage::UnexpectedError { message } => {
                eprintln!("Unexpected error: {}", message.unwrap_or("_".to_string()));
                // return None;  // don't, lets end the loop and see if it will retry.
            },
            QueueDataMessage::Unknown(m) => {
                eprintln!("[warning] Skipping unknown message: {:?}", m);
            },
        }
    }
    None
}

// Dear kids, if you don't know what the capital of japan is doing in the code, it's a little thingy to run async functions.
#[tokio::main]
async fn main() {
    println!("Whisper Large V3 turbo");
    let whisper= WhisperLarge::new(gradio::ClientOptions::default()).await.unwrap();
    // warning! This video, rust in 100 seconds, is somewhere transcribed incorrectly by whisper-jax, especially the name of the rust founder.
    let mut result=whisper.predict_background("wavs/english.wav", "transcribe").await.unwrap();
    let result=show_progress(&mut result).await;
    match &result {
        Some(result) => {
            let result=result[0].clone().as_value().unwrap();
            fs::write("result.txt", format!("{}", result)).expect("Can't write to file");
            println!("result written to result.txt");
        },
        None => {
            println!("Failed to transcribe");
        }
    }
    

}
