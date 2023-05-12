use proc_macro::TokenStream;
use proc_macro2::Group;
use proc_macro_error::*;
use syn::{spanned::Spanned, Attribute, ImplItem, ItemImpl, Lit, LitStr};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn cucumber_world(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemImpl = match syn::parse(item) {
        Ok(item) => item,
        Err(_) => abort_call_site!("This attribute is only supported on `impl` blocks"),
    };

    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();

    let input_ident = match input.self_ty.as_ref() {
        syn::Type::Path(p) if p.qself.is_none() => p,
        _ => abort_call_site!("This attribute is only supported on `impl` blocks for a path."),
    };

    let ident = input_ident.path.segments.iter().map(|i| &i.ident);
    let leading_colon = input_ident.path.leading_colon;
    let impl_ident = quote::quote! { #leading_colon #(#ident)::* };

    let items = input.items;

    items
        .iter()
        .filter(|i| !matches!(i, ImplItem::Method(_)))
        .for_each(|i| {
            emit_error!(
                i.span(),
                "`cucumber_world` only supports `fn`s in the impl block"
            )
        });

    let fns = items.iter().filter_map(|i| {
        if let ImplItem::Method(method) = i {
            Some(method)
        } else {
            None
        }
    });

    let given = fns
        .clone()
        .filter_map(|f| find_literal_attr("given", &f.attrs).map(|str| (str, f)));

    let whens = fns
        .clone()
        .filter_map(|f| find_literal_attr("when", &f.attrs).map(|str| (str, f)));

    let fns_stripped_attrs = fns.cloned().map(|mut f| {
        let new_attrs = f
            .attrs
            .iter()
            .cloned()
            .filter(|a| !attr_with_name("given", a) && !attr_with_name("when", a));
        f.attrs = new_attrs.collect();
        f
    });

    let result = quote::quote! {
        impl #impl_gen #impl_ident #ty_gen #where_clause {
            #(#fns_stripped_attrs)*
        }
    }
    .into();

    result
}

fn attr_with_name(name: &str, attr: &Attribute) -> bool {
    if attr.path.segments.len() == 1 {
        attr.path
            .segments
            .iter()
            .next()
            .map(|p| p.ident.to_string() == name)
            .unwrap_or(false)
    } else {
        false
    }
}

fn find_literal_attr(name: &str, attrs: &[Attribute]) -> Option<LitStr> {
    if attrs.iter().filter(|a| attr_with_name(name, a)).count() > 0 {
        attrs
            .iter()
            .filter(|a| attr_with_name(name, a))
            .for_each(|a| {
                let message = format!("Only one `{name}` attribute is supported per `fn`");
                emit_error!(a.span(), message)
            });
        return None;
    }

    let (ident, tokens) = attrs
        .iter()
        .find(|a| attr_with_name(name, a))
        .map(|a| a.path.segments.iter().next().map(|s| (&s.ident, &a.tokens)))??;

    let literal: LitStr = match syn::parse2::<Group>(tokens.clone()) {
        Ok(group) => match syn::parse2::<Lit>(group.stream()) {
            Ok(Lit::Str(lit)) => lit,
            _ => {
                let message = format!("The {name} attribute takes one string literal as argument");
                abort!(ident.span(), message)
            }
        },
        _ => {
            let message = format!("The {name} attribute takes one string literal as argument");
            abort!(ident.span(), message)
        }
    };

    Some(literal)
}
