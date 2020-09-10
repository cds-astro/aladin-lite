#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

#[proc_macro_derive(Shaderize, attributes(uniform, VertexShader, FragmentShader))]
pub fn derive_shader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_shaderize(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_shaderize(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let attrs = &ast.attrs;

    quote! {}
}

