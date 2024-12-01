/// Attribute macro for defining the entry point.
///
/// You can use this to annotate a struct as the game type, e.g.
///
/// ```
/// #[teenygame::game]
/// struct Game;
///
/// impl teenygame::Game for Game {
///    // Implementation goes here.
/// }
/// ```
#[proc_macro_attribute]
pub fn game(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut crate_name = syn::Ident::new("teenygame", proc_macro::Span::call_site().into());

    let args_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("crate") {
            crate_name = meta.value()?.parse()?;
            Ok(())
        } else {
            Err(meta.error("unsupported property"))
        }
    });

    syn::parse_macro_input!(args with args_parser);
    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    let name = &input.ident;
    proc_macro::TokenStream::from(quote::quote! {
        pub fn main() {
            #crate_name::run::<#name>();
        }

        #input
    })
}
