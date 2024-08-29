// This is a simple example of how to use the generated API
// The api is generated in compile time, iliminating the need to learn the API and the need to write the boilerplate code
// One simple macro fetches the api using the gradio crate and generates the api for you.
// notice 1: You need to be online to build the project for the first time, because the macro fetches the api from the server.
// notice 2: for some reason, the macro relies on serde and serde_json crate, so cargo add them to the project where you use the macro.
// we highly recommend to use an IDE that supports rust analyzer to determin the api struct name and the available methods since the api is generated in compile time.

use gradio_macro::gradio_api;
use serde_json::Value;

// The macro generates the api struct for you, so you don't need to write the struct yourself.
gradio_api!{"hf-audio/whisper-large-v3"}


fn main() {
    // this struct is autonamed by the macro. To get the name, start typing the word "Api" and the IDE will show you the name.
    let whisper= ApiHfAudioWhisperLargeV3::new().unwrap();
    let result=whisper.predict("english.wav", "transcribe").unwrap()[0].clone().as_value().unwrap();
    match &result {
        Value::String(val) => println!("{}", val),
        _ => panic!("Unexpected result: {:?}", &result.clone()),
    }
    
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
