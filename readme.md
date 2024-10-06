# gradio_macro_test

a simple macro that generates api code for gradio rust crate.

## Usage

First, to run the macro, you need to have the gradio_macro crate in your project. Add it as path, no git repo yet.

```toml
[dependencies]
gradio_macro = { path = "path/to/gradio_macro" }
# you will also need serde and serde_json, but later we will remove this dependency
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Then, you can use the macro in your code.
```rust
use gradio_macro::gradio_api;

// The macro generates the api struct for you, so you don't need to write the struct yourself.
#[gradio_api(url = "hf-audio/whisper-large-v3-turbo", option = "async")]
pub struct WhisperLarge;

#[tokio::main]
async fn main() {
    println!("Whisper Large V3 turbo");
    let whisper= WhisperLarge::new(gradio::ClientOptions::default()).await.unwrap();
    let mut result=whisper.predict("english.wav", "transcribe").await.unwrap();
    match &result {
        Ok(result) => {
            let result=result[0].clone().as_value().unwrap();
            fs::write("result.txt", format!("{}", result)).expect("Can't write to file");
            println!("result written to result.txt");
        },
        Err(e) => {
            println!("Failed to transcribe: {}", e);
        }
    }
}

## How it works

The struct uses the [gradio](https://crates.io/crates/gradio) crate to generate the api struct. The #[gradio_api(...)] attribute macro generates the struct for you, so you don't need to write it yourself.
#[gradio_api(...)] attribute has a url argument and an option argument. Url can be either full url or a simple huggingface space identifier. Option argument can be "sync" or "async", specify this based on your codebase.
The methods of the struct are just snake_cased endpoint names, with an optional "_background" prefix. Background methods are run in background using async tasks. The arg names are also got from the spec.

## Limitations

The prediction outputs are a bit complex to handle due to there dynamic nature. The macro uses serde_json to handle the outputs, but it can be improved later.

## Credits

Big Thanks to [Jacob Lin](https://github.com/JacobLinCool) for the idea and the help.

## Notes

Sounds folder of the repository has a wav'ed video of youtube channel Fireship about the latest news / code report. This file is named english.wav and put there for voice recognition test. If you found it as a copiright violation, please kindly make an issue and I will replace it.