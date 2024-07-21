extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn check_roles(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments into a list of nested meta items (key-value pairs, paths, etc.)
    // This allows us to inspect and extract the roles specified in the macro attribute.
    let args = parse_macro_input!(attr as AttributeArgs);

    // Parse the input tokens into a syntax tree representing the function.
    // This enables us to manipulate and generate new function code.
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function's visibility (e.g., pub), name, return type, inputs, body, and attributes.
    // We need these details to reconstruct the function with additional logging.
    let fn_vis = &input.vis;
    let fn_name = &input.sig.ident;
    let fn_return_type = &input.sig.output;
    let fn_inputs = &input.sig.inputs;
    let fn_body = &input.block;
    let fn_attrs = &input.attrs;

    // Extract roles from the parsed attribute arguments.
    // We look for key-value pairs where the key is "role" and the value is a string.
    let mut roles = Vec::new();
    for arg in args {
        // Check if the argument is a key-value pair.
        if let NestedMeta::Meta(Meta::NameValue(meta_name_value)) = arg {
            // Check if the key is "role".
            if meta_name_value.path.is_ident("role") {
                // Check if the value is a string literal and add it to the roles vector.
                if let Lit::Str(lit_str) = meta_name_value.lit {
                    roles.push(lit_str.value());
                }
            }
        }
    }

    // Convert roles to a format that can be used in the quote! macro.
    // This allows us to inject the roles into the generated function code.
    let roles_tokens: Vec<_> = roles.iter().map(|role| quote! { #role }).collect();

    // Generate code to log jwt_guard if it is present among the function's arguments.
    // This ensures we log the jwt_guard's roles if it is passed to the function.
    let jwt_guard_logging = fn_inputs.iter().filter_map(|arg| {
        // Check if the argument is a typed pattern (i.e., has a type specified).
        if let syn::FnArg::Typed(pat_type) = arg {
            // Check if the pattern is an identifier (i.e., a simple variable name).
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                // Check if the identifier name is "jwt_guard".
                if pat_ident.ident == "jwt_guard" {
                    return Some(quote! {
                        // Generate code to print the roles from jwt_guard.
                        println!("jwt_guard roles: {:?}", jwt_guard.claims.roles);
                    });
                }
            }
        }
        None
    });

    // Generate the new function code with logging.
    let expanded = quote! {
        // Reapply all existing attributes to the function.
        // This ensures that other attributes (e.g., routing) are preserved.
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#fn_inputs) #fn_return_type {
            // Print the roles extracted from the macro attribute.
            // This provides visibility into the roles specified at the macro level.
            println!("Roles: {:?}", vec![#(#roles_tokens),*]);

            // Print entering function message.
            // This helps track when the function is entered.
            println!("Entering function {}", stringify!(#fn_name));

            // Include jwt_guard logging code if jwt_guard is present among the function's arguments.
            // This logs the roles within jwt_guard for further inspection.
            #(#jwt_guard_logging)*

            // Execute the original function body and capture the result.
            // This ensures the function's original behavior is preserved.
            let result = (|| #fn_body)();

            // Print exiting function message along with the result.
            // This helps track when the function exits and what it returns.
            println!("Exiting function {} with result {:?}", stringify!(#fn_name), result);

            // Return the original result without modification.
            // This preserves the original functionality and return value of the function.
            result
        }
    };

    // Convert the generated code into a TokenStream and return it.
    // This is required to output the modified function back to the compiler.
    TokenStream::from(expanded)
}
