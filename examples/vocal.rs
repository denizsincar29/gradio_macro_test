// This is a simple example of how to use the generated API
// The api is generated in compile time, iliminating the need to learn the API and the need to write the boilerplate code
// One simple macro fetches the api using the gradio crate and generates the api for you.
// notice 1: You need to be online to build the project for the first time, because the macro fetches the api from the server.
// notice 2: for some reason, the macro relies on serde and serde_json crate, so cargo add them to the project where you use the macro.
// we highly recommend to use an IDE that supports rust analyzer to determin the api struct methods since the api is generated in compile time.

use gradio::{structs::QueueDataMessage, PredictionOutput, PredictionStream};
use gradio_macro::gradio_api;


// The macro generates the api struct for you, so you don't need to write the struct yourself.
#[gradio_api(url = "jacoblincool/vocal-separation", option = "async")]
pub struct VocalSeparation;

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


async fn download_file(file: gradio::GradioFileData, filename: &str) {
    tokio::fs::write(filename, file.download(None).await.unwrap()).await.unwrap();
    println!("File downloaded: {}", filename);
}


// Dear kids, if you don't know what the capital of japan is doing in the code, it's a little thingy to run async functions.
#[tokio::main]
async fn main(){
    println!("Vocal Separation");
    println!("Warning. This example doesn't work for me. The api doesn't work fine and refuses to separate vocals.");
    let vocal = VocalSeparation::new(gradio::ClientOptions::default()).await.unwrap();
    let mut task = vocal.separate_background("wavs/tunisia.wav", "BS-RoFormer").await.unwrap();
    let result = show_progress(&mut task).await.expect("Failed to separate vocals");
    // the result is vec of files. 0 is vocals, 1 is background
    let vocals = result[0].clone().as_file().unwrap();
    let background = result[1].clone().as_file().unwrap();
    // we are true async programmers, right? so we will write the files in parallel
    let vocals_task = tokio::spawn(download_file(vocals, "vocals.wav"));
    let background_task = tokio::spawn(download_file(background, "background.wav"));
    let _ = tokio::join!(vocals_task, background_task);
    
    

}
