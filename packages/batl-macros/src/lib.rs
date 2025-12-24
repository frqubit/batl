extern crate proc_macro;

use proc_macro::TokenStream as ProcMacroStream;
use proc_macro_error::proc_macro_error;
use quote::{quote, ToTokens};
use semver::Version;
use syn::{Ident, LitStr};

mod formats;
use formats::VersionedIdentical;

#[proc_macro_error]
#[proc_macro]
pub fn versioned_identical(input: ProcMacroStream) -> ProcMacroStream {
    let data = syn::parse_macro_input!(input as VersionedIdentical);
    data.into_token_stream().into()
}

#[proc_macro_error]
#[proc_macro]
pub fn semver_struct_impl(input: ProcMacroStream) -> ProcMacroStream {
    let in_litstr = syn::parse_macro_input!(input as LitStr);
    let in_string = in_litstr.value();
    let maybe_semver = Version::parse(&in_string);

    if let Ok(in_semver) = maybe_semver {
        let semver_ident_str = format!(
            "{}_{}_{}",
            in_semver.major, in_semver.minor, in_semver.patch
        );

        let version_ident = Ident::new(&format!("Version{semver_ident_str}"), in_litstr.span());

        let example_litstr = LitStr::new(&format!("`{in_string}`"), in_litstr.span());

        quote!{
			#[derive(Clone, Default, PartialEq, Eq)]
			#[allow(clippy::exhaustive_structs)]
			pub struct #version_ident;

			#[allow(clippy::missing_trait_methods)]
			impl<'de> ::serde::de::Deserialize<'de> for #version_ident {
				#[inline]
				fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
				where
					D: ::serde::de::Deserializer<'de>
				{
					/// Visitor for Version 0.2.2
					struct VersionVisitor;

					impl<'de> ::serde::de::Visitor<'de> for VersionVisitor {
						type Value = #version_ident;

						fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
							formatter.write_str(#example_litstr)
						}

						fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
						where
							E: ::serde::de::Error
						{
							match v {
								#in_litstr => ::core::result::Result::Ok(#version_ident),
								_ => ::core::result::Result::Err(::serde::de::Error::invalid_value(::serde::de::Unexpected::Str(v), &#in_litstr))
							}
						}
					}

					deserializer.deserialize_str(VersionVisitor)
				}
			}

			impl ::serde::ser::Serialize for #version_ident {
				#[inline]
				fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
				where
					S: ::serde::ser::Serializer
				{
					serializer.serialize_str(#in_litstr)
				}
			}
		}.into()
    } else {
        proc_macro_error::abort!(in_litstr, "Input is not semver");
    }
}

#[proc_macro]
pub fn environment_struct_impl(input: ProcMacroStream) -> ProcMacroStream {
    let in_litstr = syn::parse_macro_input!(input as LitStr);
    let in_string = in_litstr.value();
    let maybe_semver = Version::parse(&in_string);

    if let Ok(in_semver) = maybe_semver {
        let semver_ident_str = format!(
            "{}_{}_{}",
            in_semver.major, in_semver.minor, in_semver.patch
        );

        let version_ident = Ident::new(&format!("Version{semver_ident_str}"), in_litstr.span());

        let environ_ident = Ident::new(&format!("Environment{semver_ident_str}"), in_litstr.span());

        quote! {
            #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
            pub struct #environ_ident {
                pub version: crate::version::#version_ident
            }
        }
        .into()
    } else {
        proc_macro_error::abort!(in_litstr, "Input is not semver");
    }
}
