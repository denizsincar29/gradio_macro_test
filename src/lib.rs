use gradio::ClientOptions;
use heck::ToSnakeCase;
use proc_macro2::{Ident, Span};
use proc_macro::TokenStream;
use syn::{parse_macro_input, punctuated::Punctuated, Expr, ItemStruct, Meta};
use quote::quote;


enum Syncity {
    Sync,
    Async,
}

fn make_compile_error(message: &str) -> TokenStream {
    syn::Error::new(Span::call_site(), message).to_compile_error().into()
}

/// A procedural macro for generating API client structs and methods for interacting with a Gradio-based API.
///
/// This macro generates a client struct for the specified Gradio API, along with methods to call the API endpoints
/// synchronously or asynchronously, depending on the provided option.
///
/// # Macro Parameters
///
/// - `url`: **Required**. The base URL of the Gradio API. This is the endpoint that the generated client will interact with.
/// - `option`: **Required**. Specifies whether the generated API methods should be synchronous or asynchronous.
///   - `"sync"`: Generates synchronous methods for interacting with the API.
///   - `"async"`: Generates asynchronous methods for interacting with the API.
/// - `hf_token` (optional): huggingface space token.
/// - `auth_username` (optional): huggingface username.
/// - `auth_password` (optional): huggingface password.
///
/// # Usage
///
/// The macro will generate the API struct and methods for you automatically, so you don't need to manually define the struct.
///
/// ```rust
/// use gradio_macro::gradio_api;
///
/// // Define the API client using the macro
/// #[gradio_api(url = "hf-audio/whisper-large-v3-turbo", option = "async")]
/// pub struct WhisperLarge;
///
/// #[tokio::main]
/// async fn main() {
///     println!("Whisper Large V3 turbo");
///
///     // Instantiate the API client
///     let whisper = WhisperLarge::new().await.unwrap();
///
///     // Call the API's predict method with input arguments
///     let mut result = whisper.predict("wavs/english.wav", "transcribe").await.unwrap();
///
///     // Handle the result
///     let result = result[0].clone().as_value().unwrap();
///     std::fs::write("result.txt", format!("{}", result)).expect("Can't write to file");
///     println!("result written to result.txt");
/// }
/// ```
///
/// This example shows how to define and use an asynchronous client with the `gradio_api` macro. 
/// The API methods will be generated automatically, and you can call them using `.await` to handle asynchronous responses.
///
/// # Generated Methods
///
/// - For each API endpoint, an asynchronous method is generated that returns a `Result` wrapped in a `Future`.
/// - You can await the result of these methods and handle success or failure as shown in the example.
#[proc_macro_attribute]
pub fn gradio_api(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as ItemStruct);
    let (mut url, mut option, mut grad_token, mut grad_login, mut grad_password) = (None, None, None, None, None);

    // Parsing macro arguments
    for item in args.iter() {
        let Ok(meta_value) = item.require_name_value() else {continue;};
        let Expr::Lit(ref lit_val) = meta_value.value else {continue;};
        let syn::Lit::Str(ref lit_val) = lit_val.lit else {continue;};
        let arg_value = lit_val.value();
        if item.path().is_ident("url") {
            url = Some(arg_value);
        } else if item.path().is_ident("option") {
            option = Some(if arg_value == "sync" { Syncity::Sync } else { Syncity::Async });
        } else if item.path().is_ident("hf_token") {
            grad_token = Some(arg_value);
        } else if item.path().is_ident("auth_username") {
            grad_login = Some(arg_value);
        } else if item.path().is_ident("auth_password") {
            grad_password = Some(arg_value);
        }
    }

    // Check if url is provided
    if url.is_none() {
        return make_compile_error("url is required");
    }
    let mut grad_opts = ClientOptions::default();
    let mut grad_auth = None;
    if grad_token.is_some() {
        grad_opts.hf_token = grad_token.clone();
    }
    if grad_login.is_some() ^ grad_password.is_some() {
        return make_compile_error("Both login and password must be present!");
    } else if grad_login.is_some() && grad_password.is_some() {
        grad_auth = Some((grad_login.clone().unwrap(), grad_password.clone().unwrap()));
        grad_opts.auth = grad_auth.clone();
    }

    // Check if option is provided
    let Some(option) = option else {
        return make_compile_error("option is required");
    };

    // Fetching the API data
    let client = gradio::Client::new_sync(&(url.clone().unwrap()[..]), grad_opts).unwrap();
    let api = client.view_api().named_endpoints;

    //  generating the client options identifiers
    let grad_auth_ts = if grad_auth.is_some() {
        quote! {Some((#grad_login, #grad_password))}
    } else { quote!{None}};
    let grad_token_ts = if let Some(val) = grad_token {
        quote! {Some(#val)}
    } else { quote!{None}};
    let grad_opts_ts = quote! {
        gradio::ClientOptions {
            auth: #grad_auth_ts,
            hf_token: #grad_token_ts
        }
    };


    // Generating the functions for the API
    let mut functions: Vec<proc_macro2::TokenStream> = Vec::new();
    for (name, info) in api.iter() {
        let method_name = Ident::new(&(name.to_snake_case()), Span::call_site());
        let background_name = Ident::new(&format!("{}_background", name.to_snake_case()), Span::call_site());

        let (args, args_call): (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) = info.parameters.iter().enumerate().map(|(i, arg)| {
            let (_arg_name, arg_ident) = match &arg.label {
                Some(label) => (label.clone(), Ident::new(&label.to_snake_case(), Span::call_site())),
                None => (format!("arg{}", i), Ident::new(&format!("arg{}", i), Span::call_site())),
            };
            let is_file = arg.python_type.r#type == "filepath";
            let arg_type: proc_macro2::TokenStream = if is_file {
                quote! { impl Into<std::path::PathBuf> }
            } else {
                quote! { impl serde::Serialize }
            };
            (quote! { #arg_ident: #arg_type },
            if is_file { quote! { gradio::PredictionInput::from_file(#arg_ident) } }
            else { quote! { gradio::PredictionInput::from_value(#arg_ident) } })
        }).collect();

        // Create sync or async functions depending on the `option`
        let function: TokenStream = match option {
            Syncity::Sync => {
                quote! {
                    pub fn #method_name(&self, #(#args),*) -> Result<Vec<gradio::PredictionOutput>, anyhow::Error> {
                        self.client.predict_sync(#name, vec![#(#args_call.into()),*])
                    }

                    pub fn #background_name(&self, #(#args),*) -> Result<gradio::PredictionStream, anyhow::Error> {
                        self.client.submit_sync(#name, vec![#(#args_call.into()),*])
                    }
                }
            },
            Syncity::Async => {
                quote! {
                    pub async fn #method_name(&self, #(#args),*) -> Result<Vec<gradio::PredictionOutput>, anyhow::Error> {
                        self.client.predict(#name, vec![#(#args_call.into()),*]).await
                    }

                    pub async fn #background_name(&self, #(#args),*) -> Result<gradio::PredictionStream, anyhow::Error> {
                        self.client.submit(#name, vec![#(#args_call.into()),*]).await
                    }
                }
            },
        }.into();

        functions.push(function.into());
    }

    // Create the struct with client
    let vis = input.vis.clone();
    let struct_name = input.ident.clone();
    let api_struct = match option {
        Syncity::Sync => {
            quote! {
                #vis struct #struct_name {
                    client: gradio::Client
                }

                impl #struct_name {
                    pub fn new_sync() -> Result<Self, ()> {
                        match gradio::Client::new_sync(#url, #grad_opts_ts) {
                            Ok(client) => Ok(Self { client }),
                            Err(_) => Err(())
                        }
                    }

                    #(#functions)*
                }
            }
        },
        Syncity::Async => {
            quote! {
                #vis struct #struct_name {
                    client: gradio::Client
                }

                impl #struct_name {
                    pub async fn new() -> Result<Self, ()> {
                        match gradio::Client::new(#url, #grad_opts_ts).await {
                            Ok(client) => Ok(Self { client }),
                            Err(_) => Err(())
                        }
                    }

                    #(#functions)*
                }
            }
        },
    };

    api_struct.into()
}
