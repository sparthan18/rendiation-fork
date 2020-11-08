use crate::utils::StructInfo;
use quote::TokenStreamExt;
use quote::{format_ident, quote};

pub fn derive_ubo_impl(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
  let s = StructInfo::new(input);
  let mut generated = proc_macro2::TokenStream::new();
  generated.append_all(derive_ubo_shadergraph_instance(&s));
  generated.append_all(derive_ubo_webgl_upload_instance(&s));
  generated.append_all(derive_ubo_nyxt_wasm_instance_impl(&s));
  generated
}

fn derive_ubo_nyxt_wasm_instance_impl(s: &StructInfo) -> proc_macro2::TokenStream {
  let struct_name = &s.struct_name;
  let instance_name = format_ident!("{}WASM", struct_name);

  let fields_wasm_getter_setter = s.map_fields(|(field_name, ty)| {
    let getter_name = format_ident!("get_{}", field_name);
    let setter_name = format_ident!("set_{}", field_name);
    quote! {
      #[wasm_bindgen(getter)]
      pub fn #getter_name(&self) -> <#ty as rendiation_math::WASMAbleType>::Type {
        self.inner.mutate_item(|d| d.#field_name).to_wasm()
      }
      #[wasm_bindgen(setter)]
      pub fn #setter_name(&mut self, value: <#ty as rendiation_math::WASMAbleType>::Type) {
        self.inner.mutate_item(|d| d.#field_name = rendiation_math::WASMAbleType::from_wasm(value))
      }
    }
  });

  quote! {
    #[cfg(feature = "nyxt")]
    use wasm_bindgen::prelude::*;

    #[cfg(feature = "nyxt")]
    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct #instance_name {
      #[wasm_bindgen(skip)]
      pub inner: nyxt_core::NyxtViewerHandledObject<nyxt_core::UniformHandleWrap<#struct_name>>,
    }

    #[cfg(feature = "nyxt")]
    impl nyxt_core::NyxtUBOWrapped for #struct_name {
      type Wrapper = #instance_name;

      fn to_nyxt_wrapper(viewer: &mut nyxt_core::NyxtViewer, handle: rendiation_ral::UniformHandle<nyxt_core::GFX, Self>) -> Self::Wrapper{
        #instance_name {
          inner: viewer.make_handle_object(nyxt_core::UniformHandleWrap(handle)),
        }
      }
    }

    #[cfg(feature = "nyxt")]
    #[wasm_bindgen]
    impl #instance_name {
      #(#fields_wasm_getter_setter)*

      #[wasm_bindgen(constructor)]
      pub fn new(viewer: &mut nyxt_core::NyxtViewer) -> Self {
        let handle = viewer.mutate_inner(|inner| {
          let default_value = #struct_name::default();
          inner.resource.bindable.uniform_buffers.add(default_value)
        });
        use nyxt_core::NyxtUBOWrapped;
        #struct_name::to_nyxt_wrapper(viewer, handle)
      }
    }

  }
}

pub fn derive_ubo_webgl_upload_instance(s: &StructInfo) -> proc_macro2::TokenStream {
  let struct_name = &s.struct_name;
  let instance_name = format_ident!("{}WebGLUniformUploadInstance", struct_name);

  let instance_fields = s.map_fields(|(field_name, ty)| {
    quote! { pub #field_name: <#ty as rendiation_webgl::WebGLUniformUploadable>::UploadInstance, }
  });

  let instance_create = s.map_fields(|(field_name, ty)| {
    let field_str = format!("{}", field_name);
    quote! { #field_name:
     < <#ty as rendiation_webgl::WebGLUniformUploadable>::UploadInstance
     as rendiation_webgl::UploadInstance<#ty> >::create(
        format!("{}", #field_str).as_str(),
        gl,
        program
     ),
    }
  });

  let instance_upload = s.map_fields(|(field_name, ty)| {
    quote! { <#ty as rendiation_webgl::WebGLUniformUploadable>::upload(&value.data.#field_name, &mut self.#field_name, renderer, resources); }
  });

  quote! {
    #[cfg(feature = "webgl")]
    pub struct #instance_name {
      #(#instance_fields)*
    }

    #[cfg(feature = "webgl")]
    impl rendiation_webgl::UploadInstance<#struct_name> for #instance_name {
      fn create(
        query_name_prefix: &str,
        gl: &rendiation_webgl::WebGl2RenderingContext,
        program: &rendiation_webgl::WebGlProgram
      ) -> Self{
        Self {
          #(#instance_create)*
        }
      }
      fn upload(
        &mut self,
        value: &rendiation_ral::UniformBufferRef<'static, rendiation_webgl::WebGL, #struct_name>,
        renderer: &mut rendiation_webgl::WebGLRenderer,
        resources: &rendiation_ral::ResourceManager<rendiation_webgl::WebGL>,
      ){
        #(#instance_upload)*
      }
    }

    #[cfg(feature = "webgl")]
    impl rendiation_webgl::WebGLUniformUploadable for #struct_name {
      type UploadValue = rendiation_ral::UniformBufferRef<'static, rendiation_webgl::WebGL, #struct_name>;
      type UploadInstance = #instance_name;
    }
  }
}

pub fn derive_ubo_shadergraph_instance(s: &StructInfo) -> proc_macro2::TokenStream {
  let struct_name = &s.struct_name;
  let shadergraph_instance_name = format_ident!("{}ShaderGraphInstance", struct_name);

  let struct_name_str = format!("{}", struct_name);
  let ubo_info_name = format_ident!("{}_UBO_INFO", struct_name);

  let ubo_info_gen = s.map_fields(|(field_name, ty)| {
    let field_str = format!("{}", field_name);
    quote! { .add_field::<#ty>(#field_str) }
  });

  let instance_fields = s.map_fields(|(field_name, ty)| {
    quote! { pub #field_name: rendiation_shadergraph::Node<#ty>, }
  });

  let instance_new = s.map_fields(|(field_name, ty)| {
    let field_str = format!("{}", field_name);
    quote! { #field_name: ubo_builder.uniform::<#ty>(#field_str), }
  });

  quote! {
    #[allow(non_upper_case_globals)]
    pub static #ubo_info_name: once_cell::sync::Lazy<rendiation_shadergraph::UBOMetaInfo> =
    once_cell::sync::Lazy::new(|| {
        rendiation_shadergraph::UBOMetaInfo::new(
          #struct_name_str,
        )
        #(#ubo_info_gen)*
        .gen_code_cache()
    });

    pub struct #shadergraph_instance_name {
      #(#instance_fields)*
    }

    impl rendiation_shadergraph::ShaderGraphBindGroupItemProvider for #struct_name {
      type ShaderGraphBindGroupItemInstance = #shadergraph_instance_name;
      fn create_instance<'a>(
        name: &'static str, // uniform buffer group not need set name
        bindgroup_builder: &mut rendiation_shadergraph::ShaderGraphBindGroupBuilder<'a>,
        stage: rendiation_ral::ShaderStage)
       -> Self::ShaderGraphBindGroupItemInstance {

        let mut ubo_builder = rendiation_shadergraph::UBOBuilder::new(
          &#ubo_info_name,
          bindgroup_builder
        );

        let instance = Self::ShaderGraphBindGroupItemInstance {
          #(#instance_new)*
        };

        ubo_builder.ok(stage);
        instance
      }
    }

    impl rendiation_ral::UBOData for #struct_name {}

    #[cfg(feature = "webgpu")]
    impl rendiation_webgpu::WGPUUBOData for #struct_name {}
  }
}