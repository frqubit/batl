extern crate proc_macro;

use proc_macro::TokenStream as ProcMacroStream;
use proc_macro_error::proc_macro_error;
use quote::{quote, quote_spanned, ToTokens};
use semver::Version;
use syn::spanned::Spanned;
use syn::{Attribute, Ident, LitStr, Meta};

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

fn attr_is_lua_serde(attr: &Attribute) -> bool {
    match attr.meta.clone() {
        Meta::Path(p) => {
            if let Some(ident) = p.get_ident() {
                if ident.to_string() == "lua_serde" {
                    return true;
                }
            }

            return false;
        }
        _ => return false,
    }
}

#[proc_macro_error]
#[proc_macro_derive(ToFromLuaValue, attributes(lua_serde))]
pub fn derive_to_from_lua_value(input: ProcMacroStream) -> ProcMacroStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let input_span = input.span();
    let name = input.ident;
    let name_as_lua_str = LitStr::new(&format!("{name} as lua"), name.span());
    let name_str = LitStr::new(&format!("{name}"), name.span());

    if let syn::Data::Struct(struct_data) = input.data {
        if let syn::Fields::Named(named_fields) = struct_data.fields {
            let into_fields = named_fields
                .named
                .iter()
                .filter_map(|v| {
                    let is_lua_serde = v.attrs.iter().position(attr_is_lua_serde).is_some();
                    v.ident.clone().map(|ident| (ident, is_lua_serde))
                })
                .map(|(ident, is_lua_serde)| {
                    let ident_str = syn::LitStr::new(&ident.to_string(), ident.span());

                    if is_lua_serde {
                        quote_spanned! {ident.span() =>
                        	out_table.set(#ident_str, <::mlua::Lua as ::mlua::LuaSerdeExt>::to_value(lua, &self.#ident)?)?;
                        }
                    } else {
                        quote_spanned! {ident.span() =>
                            out_table.set(#ident_str, self.#ident)?;
                        }
                    }
                })
                .collect::<Vec<_>>();

            let into_lua_fn = quote_spanned! {input_span =>
                impl ::mlua::IntoLua for #name {
                    fn into_lua(self, lua: &::mlua::Lua) -> ::mlua::Result<::mlua::Value> {
                        let out_table = lua.create_table()?;

                        #(
                            #into_fields
                        )*

                        Ok(<::mlua::Table as ::mlua::ObjectLike>::to_value(&out_table))
                    }
                }
            };

            let from_fields = named_fields
                .named
                .iter()
                .filter_map(|v| {
                    let is_lua_serde = v.attrs.iter().position(attr_is_lua_serde).is_some();
                    v.ident.clone().map(|ident| (ident, is_lua_serde))
                })
                .map(|(ident, is_lua_serde)| {
                    let ident_str = syn::LitStr::new(&ident.to_string(), ident.span());

                    if is_lua_serde {
                        quote_spanned! {ident.span() =>
                        	#ident: <::mlua::Lua as ::mlua::LuaSerdeExt>::from_value(lua, value.get(#ident_str)?)?
                        }
                    } else {
	                    quote_spanned! {ident.span() =>
	                        #ident: ::mlua::FromLua::from_lua(value.get(#ident_str)?, lua)?
	                    }
                    }
                })
                .collect::<Vec<_>>();

            let from_lua_fn = quote_spanned! {input_span =>
                impl ::mlua::FromLua for #name {
                    fn from_lua(value: ::mlua::Value, lua: &::mlua::Lua) -> ::mlua::Result<Self> {
                        if let Some(value) = value.as_table() {
                            Ok(Self {
                                #(#from_fields),*
                            })
                        } else {
                            Err(::mlua::Error::FromLuaConversionError {
                                 from: #name_as_lua_str,
                                 to: #name_str.to_string(),
                                 message: Some(format!("must be table but was actually {}", value.type_name()))
                             })
                        }
                    }
                }
            };

            quote! {
                #into_lua_fn
                 #from_lua_fn
            }
            .into()
        } else {
            proc_macro_error::abort!(input_span, "input must be named struct values");
        }
    } else {
        proc_macro_error::abort!(input_span, "input must be struct");
    }
}
