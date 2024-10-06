# gradio_macro_test

a simple macro that generates api code for gradio rust crate.

## Usage

First, to run the macro, you need to have the gradio_macro crate in your project.

```toml
[dependencies]
gradio_macro = "0.2"
# You will also need serde and serde_json for handling API data, though this may be removed in the future.
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Then, you can use the macro in your code like this.
```rust
use gradio_macro::gradio_api;
use std::fs;

/// Define the API client using the macro
#[gradio_api(url = "hf-audio/whisper-large-v3-turbo", option = "async")]
pub struct WhisperLarge;

#[tokio::main]
async fn main() {
    println!("Whisper Large V3 turbo");

    // Instantiate the API client
    let whisper = WhisperLarge::new().await.unwrap();

    // Call the API's predict method with input arguments
    let result = whisper.predict("wavs/english.wav", "transcribe").await.unwrap();

    // Handle the result
    let result = result[0].clone().as_value().unwrap();

    // Save the result to a file
    std::fs::write("result.txt", format!("{}", result)).expect("Can't write to file");
    println!("result written to result.txt");
}
```

This example demonstrates how to define an asynchronous API client using the `gradio_api` macro to interact with the `hf-audio/whisper-large-v3-turbo` Gradio model.

### explanation

In this example, the macro defines the WhisperLarge struct, which can be used to send requests to the API. The macro also generates methods for each API endpoint based on the provided URL.
- The predict method in this case is used to send an audio file (wavs/english.wav) to the API for transcription.
- The API responds with the transcription, which is then written to result.txt.

## How it works

The struct uses the [gradio](https://crates.io/crates/gradio) crate to generate the api struct.
The #[gradio_api(...)] attribute macro generates the struct for you, so you don't need to write it yourself.
- The #[gradio_api(...)] attribute has two arguments: url and option, and 3 optional string arguments: hf_token, auth_username and auth_password for authorization on huggingface (experimental).
- url can be either a full URL or a simple Hugging Face space identifier.
- option can be "sync" or "async", depending on the nature of your codebase.
- hf_token (optional) - huggingface token.
- auth_username and auth_password (optional) - your huggingface credentials.
- The methods of the struct are snake_cased versions of the API endpoint names, with an optional _background suffix for methods that are run in the background using async tasks.
- The argument names are derived from the API spec.


## Limitations

- The prediction outputs are somewhat complex to handle due to their dynamic nature. The macro currently uses serde_json to manage the outputs, but this can be improved in future versions.

## Credits

Big Thanks to [Jacob Lin](https://github.com/JacobLinCool) for the idea and assistance.

## Notes

The sounds folder in the repository contains a .wav version of a video from the YouTube channel Fireship, covering the latest tech news/code report. This file, named english.wav, is provided for voice recognition testing. If you believe this constitutes copyright infringement, please kindly open an issue, and I will replace it.