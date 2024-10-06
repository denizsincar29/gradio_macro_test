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

#[proc_macro_attribute]
pub fn gradio_api(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as ItemStruct);
    let (mut url, mut option) = ("".to_string(), None);

    // Parsing macro arguments
    for item in args.iter() {
        let Ok(meta_value) = item.require_name_value() else {continue;};
        let Expr::Lit(ref lit_val) = meta_value.value else {continue;};
        let syn::Lit::Str(ref lit_val) = lit_val.lit else {continue;};
        let arg_value = lit_val.value();
        if item.path().is_ident("url") {
            url = arg_value;
        } else if item.path().is_ident("option") {
            option = Some(if arg_value == "sync" { Syncity::Sync } else { Syncity::Async });
        }
    }

    // Check if url is provided
    if url.is_empty() {
        return make_compile_error("url is required");
    }

    // Check if option is provided
    let Some(option) = option else {
        return make_compile_error("option is required");
    };

    // Fetching the API data
    let client = gradio::Client::new_sync(&(url[..]), gradio::ClientOptions::default()).unwrap();
    let api = client.view_api().named_endpoints;

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
                    pub fn new_sync(options: gradio::ClientOptions) -> Result<Self, ()> {
                        match gradio::Client::new_sync(#url, options) {
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
                    pub async fn new(options: gradio::ClientOptions) -> Result<Self, ()> {
                        match gradio::Client::new(#url, options).await {
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
