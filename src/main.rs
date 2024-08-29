// a speech recognition program that takes a wav file as input and outputs the recognized text
// Now possible! Gradio client is now available for Rust! Developers added sync support after my issue was opened.

#[cfg(feature = "gen_api")]
use std::{fs::File, io::Write};
use gradio_macro::gradio_api;

// lets move api prety writing to a function. It takes a client and file to write to
#[cfg(feature = "gen_api")]
fn write_api(client: &Client, mut f: File) {
    let api=client.view_api().named_endpoints;
    // OMG, the api is a hashmap of endpoint names and endpoint info objects that i don't know how to use. Lets iterate over it
    for (name, info) in api.iter() {
        writeln!(f, "# Endpoint: {}", name);
        writeln!(f, "args:").unwrap();
        for arg in info.parameters.clone() {
            // let's check line by line what is in the arg object, because everything is optional. Lets 2space indent the label, and 4 space indent everything else
            writeln!(f, "- {}: {}({}, {})", arg.label.unwrap_or("Unnamed".into()), arg.parameter_name.unwrap_or("".into()), arg.r#type.r#type, arg.r#type.description).unwrap();
            if let Some(default)=arg.parameter_default {
                writeln!(f, "    - default: {}", default).unwrap();
            }
            writeln!(f, "    - component: {}", arg.component).unwrap();  // this is a string, but i don't know what it means
            writeln!(f, "Example:").unwrap();
            writeln!(f, "```").unwrap();
            // let jvalue=arg.example_input.unwrap_or(Value.into());  // nope, we will use if let.
            if let Some(val) = arg.example_input {
                // val is serde json value. OMG lets just turn it into a string if derived debug or display
                writeln!(f, "{}", val).unwrap();

            }
        
            writeln!(f, "\n```\n").unwrap();
            
        }
    }
}

fn main() {
    // huggingface model sanchit-gandhi/whisper-jax

    gradio_api!{"hf-audio/whisper-large-v3"};
    let whisper= ApiHfAudioWhisperLargeV3::new().unwrap();
    let result=whisper.predict("english.wav", "transcribe").unwrap();
    println!("{:?}", result[0]);
    
    /*
    // if the result is enum value gradio::PredictionOutput::Value and inside another enum serde json value string, than it's great. Otherwise error
    if let PredictionOutput::Value(Value::String(val))=&result[0] {
        println!("{}", val);
    } else {
        panic!("Unexpected result: {:?}", result);  // OMG! What did whisper-large-v3 return?
    }
    println!("{:?}", result);
    */

}
