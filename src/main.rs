use gradio_macro::gradio_api;

gradio_api!{"hf-audio/whisper-large-v3"};


fn main() {
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
