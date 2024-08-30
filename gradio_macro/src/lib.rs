// attention! This macro will fetch the api data from the internet, so rust analyzer users please turn off the reevaluation of the macro in the settings

// the macro will take a string literal of the model name or url (as in the gradio client), fetch the api data using the client and make a struct with the api methods, there types and other stuff to make it simple to use the api

use proc_macro2::{Ident, Span};
use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};
use quote::quote;

fn to_camel_case(input: &str) -> String {
    // Split the input string by non-alphanumeric characters using a regex
    let re = regex::Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    let parts: Vec<&str> = re.split(input).collect();

    // Capitalize the first letter of each part and collect them into a Vec<String>
    let camel_case_parts: Vec<String> = parts.iter()
        .filter_map(|part| {
            if part.is_empty() {
                None
            } else {
                let mut capitalized = part.to_lowercase();
                capitalized[..1].make_ascii_uppercase();
                Some(capitalized)
            }
        })
        .collect();

    // Join the parts together without any spaces
    camel_case_parts.join("")
}

fn to_snake_case(input: &str) -> String {
    // Split the input string by non-alphanumeric characters using a regex
    let re = regex::Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    let parts: Vec<&str> = re.split(input).collect();

    // Lowercase the first part and join the rest with underscores
    let snake_case_parts: Vec<String> = parts.iter()
        .filter_map(|part| {
            if part.is_empty() {
                None
            } else {
                Some(part.to_lowercase())
            }
        })
        .collect();
    // Join the parts together with underscores
    snake_case_parts.join("_")
}

#[proc_macro]
pub fn gradio_api(input: TokenStream) -> TokenStream {
    let input=parse_macro_input!(input as LitStr);
    let client=gradio::Client::new_sync(&(input.value()[..]), gradio::ClientOptions::default()).unwrap();
    let api=client.view_api().named_endpoints;
    let config=client.view_config();
    // what if we use multiple hf models? we need to make struct name unique.
    let struct_name = if let Some(sid) = config.space_id {
        let ident_name=to_camel_case(&sid);
        Ident::new(&format!("Api{}", ident_name), Span::call_site())
    } else {
        Ident::new("Api", Span::call_site())
    };
    
    let mut functions: Vec<proc_macro2::TokenStream>=Vec::new(); // this will be the pub functions in the impl block as tokenstreams
    for (name, info) in api.iter() {
        // this will be the pub function in the impl block
        let method_name = Ident::new(&to_snake_case(&name), Span::call_site());
        let method_name_sync = Ident::new(&to_snake_case(&format!("{}_sync", &name)[..]), Span::call_site());
        // args is a vector of quote!{argument: type}
        let (args, args_call): (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) = info.parameters.iter().enumerate().map(|(i, arg)| {
            let (_arg_name, arg_ident) = match &arg.label {
                Some(label) => (label.clone(), Ident::new(&to_snake_case(&label), Span::call_site())),
                None => (format!("arg{}", i), Ident::new(&format!("arg{}", i), Span::call_site())),
            };
            let is_file=arg.python_type.r#type == "filepath";
            let arg_type: proc_macro2::TokenStream = if is_file {
                quote! { impl Into<std::path::PathBuf> }.into()
            } else {
                quote! { impl serde::Serialize }.into()
            };
            (quote! {
                #arg_ident: #arg_type
            },
            if is_file {quote! {
                gradio::PredictionInput::from_file(#arg_ident)
            }} else { quote! {
                gradio::PredictionInput::from_value(#arg_ident)
            }}.into())

        }).collect(); // end of args (map)
        // lets build pub fn with the args
        let function: TokenStream=quote! {
            pub fn #method_name_sync(&self, #(#args),*) -> Result<Vec<gradio::PredictionOutput>, anyhow::Error> {
                self.client.predict_sync(#name, vec![#(#args_call.into()),*])
            }
            pub async fn #method_name(&self, #(#args),*) -> Result<Vec<gradio::PredictionOutput>, anyhow::Error> {
                self.client.predict(#name, vec![#(#args_call.into()),*]).await
            }
        }.into();  // end of quote!{function}
        functions.push(function.into());
    };  // end of for loop

    // this will be the struct with the client field
    let api_struct=quote! {
        pub struct #struct_name {
            client: gradio::Client
        }
        impl #struct_name {
            pub fn new_sync(options: gradio::ClientOptions) -> Result<Self, ()> {
                match gradio::Client::new_sync(#input, options) {
                    Ok(client) => Ok(Self { client }),
                    Err(_) => Err(())
                }
            }
            // async
            pub async fn new(options: gradio::ClientOptions) -> Result<Self, ()> {
                match gradio::Client::new(#input, options).await {
                    Ok(client) => Ok(Self { client }),
                    Err(_) => Err(())
                }
            }
            #(#functions)*
        }

    };
    api_struct.into()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "HelloWorld");
        assert_eq!(to_camel_case("hello-world"), "HelloWorld");
        assert_eq!(to_snake_case("/predict_1"), "predict_1");  // api endpoint example
    }
    }
