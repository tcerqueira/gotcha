use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatType, ReturnType};

// TODO: unchatGPT this hot garbage code
#[proc_macro_attribute]
pub fn integration_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Extract function name and body
    let fn_name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    // Check if function is async
    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(&input.sig, "integration test function must be async")
            .to_compile_error()
            .into();
    }

    // Get the context parameter
    let ctx_ident = match input.sig.inputs.first() {
        Some(FnArg::Typed(PatType { pat, ty, .. })) => {
            if let Pat::Ident(pat_ident) = &**pat {
                // Verify the type is TestContext
                if !quote!(#ty).to_string().contains("TestContext") {
                    return syn::Error::new_spanned(
                        ty,
                        "first parameter must be of type TestContext",
                    )
                    .to_compile_error()
                    .into();
                }
                pat_ident.ident.clone()
            } else {
                return syn::Error::new_spanned(pat, "first parameter must be a simple identifier")
                    .to_compile_error()
                    .into();
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &input.sig,
                "function must take TestContext as first parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    // Handle return type
    let return_type = match &input.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(quote!(#ty)),
    };

    let output = if return_type.is_some() {
        quote! {
            #(#attrs)*
            #[tokio::test]
            async fn #fn_name() -> #return_type {
                gotcha_server::test_helpers::with_test_context(|#ctx_ident| async move #body).await
            }
        }
    } else {
        quote! {
            #(#attrs)*
            #[tokio::test]
            async fn #fn_name() {
                gotcha_server::test_helpers::with_test_context(|#ctx_ident| async move #body).await
            }
        }
    };

    output.into()
}
