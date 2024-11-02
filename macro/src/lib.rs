/// Attribute macro for defining the entry point.
///
/// You can use this to annotate a struct as the game type, e.g.
///
/// ```
/// #[teenygame::main]
/// struct Game;
///
/// impl teenygame::Game for Game {
///    // Implementation goes here.
/// }
/// ```
#[proc_macro_attribute]
pub fn main(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    let name = &input.ident;
    proc_macro::TokenStream::from(quote::quote! {
        pub fn main() {
            teenygame::run::<#name>();
        }

        #input
    })
}
