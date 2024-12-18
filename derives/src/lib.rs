use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(SketchComponents, attributes(sketch))]
pub fn sketch_components(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    // Get the fields if this is a struct
    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("SketchComponents only works on structs with named fields"),
    };

    let window_rect_impl = fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "window_rect")
        .map(|_| {
            quote! {
                fn window_rect(&mut self) -> Option<&mut WindowRect> {
                    Some(&mut self.window_rect)
                }
            }
        });

    let controls_impl = fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "controls")
        .map(|_| {
            quote! {
                fn controls(&mut self) -> Option<&mut Controls> {
                    Some(&mut self.controls)
                }
            }
        });

    let gen = quote! {
        impl SketchModel for #name {
            #window_rect_impl
            #controls_impl
        }
    };

    gen.into()
}
