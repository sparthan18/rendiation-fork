#[proc_macro_derive(Incremental)]
pub fn derive_incremental(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as syn::DeriveInput);
  derive_incremental_impl(&input).into()
}

use proc_macro::TokenStream;
use quote::TokenStreamExt;
use quote::{format_ident, quote};
use syn::parse_macro_input;

fn derive_incremental_impl(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
  let s = StructInfo::new(input);
  let mut generated = proc_macro2::TokenStream::new();
  generated.append_all(derive_incremental_impl_inner(&s));
  generated
}

fn derive_incremental_impl_inner(s: &StructInfo) -> proc_macro2::TokenStream {
  let struct_name = &s.struct_name;
  let incremental_type_name = format_ident!("{}Delta", struct_name);

  let incremental_variants = s.map_visible_fields(|(name, ty)| {
    quote! { #name(incremental::DeltaOf<#ty>), }
  });

  let apply = s.map_visible_fields(|(name, _)| {
    quote! {
      #incremental_type_name::#name(v) => {
        return self.#name.apply(v).map_err(|_|{});
      },
    }
  });

  let should_apply_hints = s.map_visible_fields(|(name, _)| {
    quote! {
      #incremental_type_name::#name(v) => {
        return self.#name.should_apply_hint(v);
      }
    }
  });

  let expand = s.map_visible_fields(|(name, _)| {
    quote! {self.#name.expand(|delta|{
      cb(#incremental_type_name::#name(delta));
    }); }
  });

  let expand_size = expand.len();
  let expand_size = quote! { #expand_size };

  quote! {

    #[allow(non_camel_case_types)]
    #[derive(Clone)]
    pub enum #incremental_type_name {
      #(#incremental_variants)*
    }

    impl incremental::IncrementalBase for #struct_name {
      type Delta = #incremental_type_name;
      fn expand(&self, mut cb: impl FnMut(Self::Delta)) {
        #(#expand)*
      }
      fn expand_size(&self) -> Option<usize> {
        Some(#expand_size)
      }
    }

    impl incremental::ApplicableIncremental for  #struct_name {
      type Error = ();
      fn apply(&mut self, delta: Self::Delta) -> Result<(), Self::Error> {
        match delta {
          #(#apply)*
        }
      }
      fn should_apply_hint(&self, delta: &Self::Delta) -> bool {
        match delta {
          #(#should_apply_hints)*
        }
      }
    }

  }
}

use syn::{punctuated::Punctuated, spanned::Spanned, Data, Field, Ident, Type, Visibility};
struct StructInfo {
  pub struct_name: Ident,
  pub _fields_info: Vec<(Ident, Type)>,
  pub fields_raw: Vec<Field>,
}

impl StructInfo {
  pub fn new(input: &syn::DeriveInput) -> Self {
    let struct_name = input.ident.clone();
    let fields = only_named_struct_fields(input).unwrap();
    let _fields_info = fields
      .iter()
      .map(|f| {
        let field_name = f.ident.as_ref().unwrap().clone();
        let ty = f.ty.clone();
        (field_name, ty)
      })
      .collect();

    let fields_raw = fields.iter().cloned().collect();

    StructInfo {
      struct_name,
      _fields_info,
      fields_raw,
    }
  }

  pub fn _map_fields(
    &self,
    f: impl FnMut(&(Ident, Type)) -> proc_macro2::TokenStream,
  ) -> Vec<proc_macro2::TokenStream> {
    self._fields_info.iter().map(f).collect()
  }

  pub fn map_visible_fields(
    &self,
    f: impl FnMut((Ident, Type)) -> proc_macro2::TokenStream,
  ) -> Vec<proc_macro2::TokenStream> {
    self
      .fields_raw
      .iter()
      .filter_map(|f| {
        if matches!(f.vis, Visibility::Inherited) {
          None
        } else {
          let field_name = f.ident.as_ref().unwrap().clone();
          let ty = f.ty.clone();
          (field_name, ty).into()
        }
      })
      .map(f)
      .collect()
  }
}

fn only_accept_struct(input: &syn::DeriveInput) -> Result<&syn::DeriveInput, syn::Error> {
  match &input.data {
    Data::Struct(_) => Ok(input),
    Data::Enum(e) => Err(syn::Error::new(
      e.enum_token.span(),
      "Cannot be derived from enums",
    )),
    Data::Union(u) => Err(syn::Error::new(
      u.union_token.span(),
      "Cannot be derived from unions",
    )),
  }
}

fn only_named_struct_fields(
  input: &syn::DeriveInput,
) -> Result<&Punctuated<Field, syn::token::Comma>, syn::Error> {
  only_accept_struct(input)?;
  let fields = if let syn::Data::Struct(syn::DataStruct {
    fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
    ..
  }) = input.data
  {
    named
  } else {
    return Err(syn::Error::new(
      input.span(),
      "Can only be derived from structs with named fields",
    ));
  };
  Ok(fields)
}
