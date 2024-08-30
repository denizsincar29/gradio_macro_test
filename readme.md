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
gradio_api!{"hf-audio/whisper-large-v3"}

fn main(){
     // this struct is autonamed by the macro. To get the name, start typing the word "Api" and the IDE will show you the name.
    let whisper= ApiHfAudioWhisperLargeV3::new().unwrap();
    let result=whisper.predict("english.wav", "transcribe").unwrap()[0].clone().as_value().unwrap();
    println!("{}", result);
}
```

## How it works

The struct uses the [gradio](https://crates.io/crates/gradio) crate to generate the api struct. The macro generates the struct for you, so you don't need to write it yourself. The struct is named after the model name, and the fields are named after the input and output names.
The methods of the struct are just snake_cased endpoint names. The arg names are also got from the spec.

## Limitations

The macro still doesn't have async and submit support, but it can be added easily later. Also the prediction outputs are a bit complex to handle due to there dynamic nature. The macro uses serde_json to handle the outputs, but it can be improved later.

## Credits

Big Thanks to [Jacob Lin](https://github.com/JacobLinCool) for the idea and the help.
