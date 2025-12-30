use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn scope(_args: TokenStream, function: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(function as ItemFn);
    let function_name = function.sig.ident.to_string();
    let stmts = function.block.stmts;

    function.block = parse_quote!({
        use logging::{o, FnValue, Level, scope, logger};
        use std::thread::current;

        let _guard = logging::set_level(Level::Trace);
        scope(&logger().new(o!("test_name" => #function_name, "thread_num" => FnValue(|_| format!("{:?}", current().id())))), || {
            #(#stmts)*
        });
    });

    TokenStream::from(quote!(#function))
}

#[proc_macro_attribute]
pub fn with_trace_id(_args: TokenStream, function: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(function as ItemFn);
    let stmts = function.block.stmts;

    function.block = parse_quote!({
        use logging::{o, scope, logger, new_trace_id};

        scope(&logger().new(o!("trace" => new_trace_id())), || {
            #(#stmts)*
        })
    });

    TokenStream::from(quote!(#function))
}
