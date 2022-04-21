extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};
use quote::quote_spanned;

#[proc_macro_attribute]
pub fn iai(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);

    let span = proc_macro2::Span::call_site();

    let function_name = find_name(item.clone());
    let wrapper_function_name = Ident::new(&format!("wrap_{}", function_name.to_string()), span);
    let const_name = Ident::new(&format!("IAI_FUNC_{}", function_name.to_string()), span);
    let name_literal = function_name.to_string();

    let output = quote_spanned!(span=>
        #item

        fn #wrapper_function_name(iai: &mut iai::Iai) {
            let _ = iai::black_box(#function_name(iai));
        }

        #[test_case]
        const #const_name : (&'static str, fn(&mut iai::Iai)) = (#name_literal, #wrapper_function_name);
    );

    output.into()
}

fn find_name(stream: proc_macro2::TokenStream) -> Ident {
    let mut iter = stream.into_iter();
    while let Some(tok) = iter.next() {
        if let TokenTree::Ident(ident) = tok {
            if ident == "fn" {
                break;
            }
        }
    }

    if let Some(TokenTree::Ident(name)) = iter.next() {
        name
    } else {
        panic!("Unable to find function name")
    }
}
