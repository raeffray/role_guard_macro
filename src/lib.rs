extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn log_execution(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let args = parse_macro_input!(attr as AttributeArgs);

    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function visibility, signature, and body
    let fn_vis = &input.vis;
    let fn_name = &input.sig.ident;
    let fn_return_type = &input.sig.output;
    let fn_inputs = &input.sig.inputs;
    let fn_body = &input.block;
    let fn_attrs = &input.attrs;

    // Extract roles from args and generate printing code
    let mut roles = Vec::new();
    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(meta_name_value)) = arg {
            if meta_name_value.path.is_ident("role") {
                if let Lit::Str(lit_str) = meta_name_value.lit {
                    roles.push(lit_str.value());
                }
            }
        }
    }


    // Convert roles to a format that can be quoted
    let roles_tokens: Vec<_> = roles.iter().map(|role| quote! { #role }).collect();

    // Generate code to log jwt_guard if it is present
    let jwt_guard_logging = fn_inputs.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                if pat_ident.ident == "jwt_guard" {
                    return Some(quote! {
                        println!("jwt_guard: {:?}", jwt_guard.claims.roles);
                    });
                }
            }
        }
        None
    });



    // Generate the new function code with logging
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#fn_inputs) #fn_return_type {
            println!("Roles: {:?}", vec![#(#roles_tokens),*]);
            println!("Entering function {}", stringify!(#fn_name));
            #(#jwt_guard_logging)*

            println!("Entering function {}", stringify!(#fn_name));
            let result = (|| #fn_body)(); // Execute the original function body
            println!("Exiting function {} with result {:?}", stringify!(#fn_name), result);
            result // Return the original result without modification
        }
    };

    // Convert the generated code into a TokenStream and return it
    TokenStream::from(expanded)
}
