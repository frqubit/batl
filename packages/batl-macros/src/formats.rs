use quote::{quote_spanned, ToTokens};
use syn::{bracketed, parse::Parse, Ident, LitStr, Token};

pub struct VersionedIdentical {
    old: LitStr,
    new: LitStr,
    objects: Vec<Ident>,
}

impl Parse for VersionedIdentical {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let old = input.parse()?;
        input.parse::<Token![=>]>()?;
        let new = input.parse()?;
        input.parse::<Token![:]>()?;

        let content;
        bracketed!(content in input);

        let mut objects = vec![];
        objects.push(content.parse()?);
        while !content.is_empty() {
            content.parse::<Token![,]>()?;
            objects.push(content.parse()?);
        }

        Ok(Self { old, new, objects })
    }
}

impl ToTokens for VersionedIdentical {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut old_str = self.old.value();
        let mut new_str = self.new.value();

        old_str = old_str.replace(".", "_");
        if new_str == "latest" {
            new_str = "Latest".to_string();
        } else {
            new_str = new_str.replace(".", "_");
        }

        let streams = self
            .objects
            .iter()
            .map(|object| {
                let old_ident = Ident::new(
                    &format!("{}{}", &object.to_string(), &old_str),
                    object.span(),
                );

                let new_ident = Ident::new(
                    &format!("{}{}", &object.to_string(), &new_str),
                    object.span(),
                );

                quote_spanned! {object.span() =>
                    pub type #new_ident = #old_ident;
                }
            })
            .collect::<Vec<_>>();

        tokens.extend(streams);
    }
}
