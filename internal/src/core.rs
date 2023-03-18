use std::{
    env, fs,
    path::{Path, PathBuf},
};

use darling::{ast, FromDeriveInput, FromField};
use genco::{
    prelude::{rust, Rust},
    quote, Tokens,
};

use genco::fmt;

use crate::path_visitor;

pub fn tokens_to_string(tokens: &Tokens<Rust>) -> Vec<u8> {
    let mut w = fmt::IoWriter::new(Vec::<u8>::new());

    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(4));
    let config = rust::Config::default()
        // Prettier imports and use.
        .with_default_import(rust::ImportMode::Qualified);

    if let Err(error) = tokens.format_file(&mut w.as_formatter(&fmt), &config) {
        panic!("{:?}", error);
    }

    w.into_inner()
}

pub fn generate_pretty_plain_text_from_tokens(tokens: &mut rust::Tokens) -> String {
    let vector = tokens_to_string(tokens);

    let utf = match std::str::from_utf8(vector.as_slice()) {
        Ok(utf) => utf,
        Err(error) => panic!(
            "There is a problem with converting bytes to utf8: {}",
            error
        ),
    };

    generate_pretty_plain_text(utf)
}

pub fn generate_pretty_plain_text(utf: &str) -> String {
    let syntax_tree = match syn::parse_file(utf) {
        Ok(parsed) => parsed,
        Err(error) => panic!(
            "There is a problem with parsing the generated rust code: {}",
            error
        ),
    };

    // it seems that genco's code formatting is broken on stable
    prettyplease::unparse(&syntax_tree)
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(lean_buffer_internal), supports(struct_any))]
pub struct InputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    pub data: ast::Data<(), FieldReceiver>,
}

impl InputReceiver {
    pub fn write(
        &mut self,
        dest_path: &PathBuf,
        factory_module: Option<String>,
        factory_name: Option<String>,
        as_module_alias: Option<String>,
    ) {
        let tokens = &mut rust::Tokens::new();
        self.generate_tokens(tokens, factory_module, factory_name, as_module_alias);
        let code = generate_pretty_plain_text_from_tokens(tokens);
        if let Err(error) = fs::write(&dest_path, code.as_str()) {
            panic!(
                "There is a problem writing the generated rust code: {:?}",
                error
            );
        }
    }

    pub fn write_to_out_dir(
        &mut self,
        factory_module: Option<String>,
        factory_name: Option<String>,
        as_module_alias: Option<String>,
    ) {
        if let Some(out_dir) = env::var_os("OUT_DIR") {
            let dest_path =
                Path::new(&out_dir).join(format!("{}_lb_gen.rs", self.ident.to_string().clone()));
            self.write(&dest_path, factory_module, factory_name, as_module_alias);
        } else {
            panic!("Missing OUT_DIR environment variable, add a `build.rs` with at least an empty `fn main` to the root of your project");
        }
    }

    pub fn generate_tokens(
        &self,
        tokens: &mut rust::Tokens,
        factory_module: Option<String>,
        factory_name: Option<String>,
        as_module_alias: Option<String>,
    ) {
        tokens.append(
            self.generate_factory(
                factory_module
                    .unwrap_or("lean_buffer::traits".to_string())
                    .as_str(),
                factory_name.unwrap_or("Factory".to_string()).as_str(),
                as_module_alias.unwrap_or("f".to_string()).as_str(),
            ),
        );
        tokens.append(self.generate_table_adapter());
    }

    fn generate_factory(
        &self,
        factory_module: &str,
        factory_name: &str,
        as_module_alias: &str,
    ) -> Tokens<Rust> {
        let fields = self
            .data
            .as_ref()
            .take_struct()
            .expect("Enums are not supported (yet)")
            .fields;

        let fb_table = &rust::import("flatbuffers", "Table");
        let factory =
            &rust::import(factory_module, factory_name).with_module_alias(as_module_alias);
        let factory_ext = &rust::import("lean_buffer::traits", "FactoryExt");
        let entity = &rust::import("self", &self.ident.to_string());

        let destructured_props = fields.iter().map(|p| p.as_struct_property_default());
        let assigned_props = fields
            .iter()
            .enumerate()
            .map(|p| p.1.as_assigned_property(p.0 * 2 + 4));

        quote! {
          impl $factory_ext<$entity> for $factory<$entity> {
            fn inflate<'a>(&self, table: &mut $fb_table<'a>) -> $entity {
              let mut object = self.new_object();
              // destructure
              let $entity {
                $(for f in &fields join (, ) => $(f.get_name()))
              } = &mut object;
              unsafe {
                $(for p in assigned_props join () => $(p))
              }
              object
            }

            fn new_object(&self) -> $entity {
              $entity {
                $(for p in destructured_props join (, ) => $(p))
              }
            }
          }
        }
    }

    fn generate_table_adapter(&self) -> Tokens<Rust> {
        let entity = &rust::import("self", &self.ident.to_string());
        let bridge_trait = &rust::import("lean_buffer::traits", "AdapterExt");
        let flatbuffer_builder = &rust::import("flatbuffers", "FlatBufferBuilder");

        // TODO Box or Rc instances, and define a `fn get_fields(&self) -> Vec<Rc<clone>>`
        let fields = self
            .data
            .as_ref()
            .take_struct()
            .expect("Enums are not supported (yet)")
            .fields;

        let unnested_props: Vec<Tokens<Rust>> = fields
            .iter()
            .enumerate()
            .map(|(i, p)| p.encode_flatten_unnested(i * 2 + 4))
            .collect();

        let mut props_unsorted: Vec<(usize, Tokens<Rust>)> = fields
            .iter()
            .enumerate()
            .map(|(i, p)| (p.to_sorting_priority(), p.encode_flatten(i * 2 + 4)))
            .collect();

        props_unsorted.sort_by(|a, b| a.0.cmp(&b.0));
        let props: Vec<Tokens<Rust>> = props_unsorted.iter().map(|t| t.1.clone()).collect();

        quote! {
          impl $bridge_trait for $entity {
            fn flatten(&self, builder: &mut $flatbuffer_builder) {
              builder.reset();
              $unnested_props
              let wip_offset_unfinished = builder.start_table();
              $props
              let wip_offset_finished = builder.end_table(wip_offset_unfinished);
              builder.finish_minimal(wip_offset_finished);
            }
          }
        }
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(lean_buffer_internal))]
pub struct FieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

impl FieldReceiver {
    fn get_name(&self) -> String {
        match &self.ident {
            Some(i) => i.to_string(),
            None => String::new(),
        }
    }

    fn as_struct_property_default(&self) -> Tokens<Rust> {
        let name = self.ident.clone().unwrap().to_string();
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if joined.starts_with("Option") {
            let r = quote! {
                $name: None
            };
            match joined.as_str() {
                "OptionString" => r,
                "Optionchar" => r,
                "Optionbool" => r,
                "Optionf32" => r,
                "Optionf64" => r,
                "Optioni8" => r,
                "Optionu8" => r,
                "Optioni16" => r,
                "Optionu16" => r,
                "Optioni32" => r,
                "Optionu32" => r,
                "Optioni64" => r,
                "Optionu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else if joined.starts_with("Vec") {
            let prim = joined.replace("Vec", "");
            let r = quote! {
                $name: Vec::<$prim>::new()
            };
            match joined.as_str() {
                "VecString" => r,
                "Vecchar" => r,
                "Vecbool" => r,
                "Vecf32" => r,
                "Vecf64" => r,
                "Veci8" => r,
                "Vecu8" => r,
                "Veci16" => r,
                "Vecu16" => r,
                "Veci32" => r,
                "Vecu32" => r,
                "Veci64" => r,
                "Vecu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else {
            let r = quote! {
                $(name.clone()): 0
            };
            match joined.as_str() {
                "String" => quote! {
                    $name: String::from("")
                },
                "char" => quote! {
                    $name: char::from(0)
                },
                "bool" => quote! {
                    $name: false
                },
                "f32" => quote! {
                    $name: 0.0
                },
                "f64" => quote! {
                    $name: 0.0
                },
                "i8" => r,
                "u8" => r,
                "i16" => r,
                "u16" => r,
                "i32" => r,
                "u32" => r,
                "i64" => r,
                "u64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        }
    }

    fn as_assigned_property(&self, offset: usize) -> Tokens<Rust> {
        let fuo = &rust::import("flatbuffers", "ForwardsUOffset");
        let fvec = &rust::import("flatbuffers", "Vector");

        let name = &self.ident.clone().unwrap().to_string();
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if joined.starts_with("Option") {
            let prim = joined.replace("Option", "");
            let r = quote! {
                *$name = table.get::<$prim>($offset, None);
            };
            match joined.as_str() {
                "OptionString" => quote! {
                    *$name = table.get::<$fuo<&str>>($offset, None).map(|s|s.to_string());
                },
                "Optionchar" => quote! {
                    if let Some(v) = table.get::<u32>($offset, None) {
                        *$name = std::char::from_u32(v);
                    }
                },
                "Optionbool" => quote! {
                    *$name = table.get::<bool>($offset, None);
                },
                "Optionf32" => quote! {
                    *$name = table.get::<f32>($offset, None);
                },
                "Optionf64" => quote! {
                    *$name = table.get::<f64>($offset, None);
                },
                "Optioni8" => r,
                "Optionu8" => r,
                "Optioni16" => r,
                "Optionu16" => r,
                "Optioni32" => r,
                "Optionu32" => r,
                "Optioni64" => r,
                "Optionu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else if joined.starts_with("Vec") {
            let prim = joined.replace("Vec", "");
            let r = quote! {
                let fb_$name = table.get::<$fuo<$fvec<$(prim.clone())>>>($offset, None);
                if let Some(v) = fb_$name {
                    *$name = v.iter().map(|s|s).collect();
                }
            };
            match joined.as_str() {
                "VecString" => quote! {
                    let fb_$name = table.get::<$fuo<$fvec<'a, $fuo<&'a str>>>>($offset, None);
                    if let Some(v) = fb_$name {
                        *$name = v.iter().map(|s|s.to_string()).collect();
                    }
                },
                "Vecchar" => quote! {
                    let fb_$name = table.get::<$fuo<$fvec<u32>>>($offset, None);
                    if let Some(c) = fb_$name {
                        *$name = c.iter().filter_map(|s| char::from_u32(s)).collect();
                    }
                },
                "Vecbool" => r,
                "Vecf32" => r,
                "Vecf64" => r,
                "Veci8" => quote! {
                    let fb_$name = table.get::<$fuo<$fvec<$prim>>>($offset, None);
                    if let Some(vb) = fb_$name {
                        let vec_u8 = vb.bytes().to_vec();
                        let slice_u8 = vec_u8.as_slice();
                        let slice_i8 = &*(slice_u8 as *const _  as *const [i8]);
                        *$name = slice_i8.to_vec();
                    }
                },
                "Vecu8" => quote! {
                    let fb_$name = table.get::<$fuo<$fvec<$prim>>>($offset, None);
                    if let Some(vb) = fb_$name {
                        *$name = vb.bytes().to_vec();
                    }
                },
                "Veci16" => r,
                "Vecu16" => r,
                "Veci32" => r,
                "Vecu32" => r,
                "Veci64" => r,
                "Vecu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else {
            let r = quote! {
                *$name = table.get::<$(joined.clone())>($offset, Some(0)).unwrap();
            };
            match joined.as_str() {
                "String" => quote! {
                    if let Some(s) = table.get::<$fuo<&str>>($offset, None) {
                        *$name = s.to_string();
                    }else {
                        *$name = "".to_string();
                    }
                },
                "char" => quote! {
                    let $(name)_u32 = table.get::<u32>($offset, Some(0)).unwrap();
                    if let Some(c) = std::char::from_u32($(name)_u32) {
                        *$name = c;
                    }
                },
                "bool" => quote! {
                    *$name = table.get::<bool>($offset, Some(false)).unwrap();
                },
                "f32" => quote! {
                    *$name = table.get::<f32>($offset, Some(0.0)).unwrap();
                },
                "f64" => quote! {
                    *$name = table.get::<f64>($offset, Some(0.0)).unwrap();
                },
                "i8" => r,
                "u8" => r,
                "i16" => r,
                "u16" => r,
                "i32" => r,
                "u32" => r,
                "i64" => r,
                "u64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        }
    }

    // TODO close the gap
    fn to_sorting_priority(&self) -> usize {
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if joined.starts_with("Option") {
            match joined.as_str() {
                "Optionf64" => 1,
                "Optionu64" => 1,
                "Optioni64" => 1,
                "OptionString" => 4,
                "Optionf32" => 5,
                "Optionu32" => 5,
                "Optioni32" => 5,
                "Optionchar" => 5,
                "Optionu16" => 6,
                "Optioni16" => 6,
                "Optionbool" => 7,
                "Optionu8" => 7,
                "Optioni8" => 7,
                _ => panic!("Not supported: {}", joined),
            }
        } else if joined.starts_with("Vec") {
            let r = 2;
            match joined.as_str() {
                "VecString" => r,
                "Vecchar" => r,
                "Vecbool" => r,
                "Vecf32" => r,
                "Vecf64" => r,
                "Veci8" => r,
                "Vecu8" => r,
                "Veci16" => r,
                "Vecu16" => r,
                "Veci32" => r,
                "Vecu32" => r,
                "Veci64" => r,
                "Vecu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else {
            match joined.as_str() {
                "f64" => 1,
                "u64" => 1,
                "i64" => 1,
                "String" => 4,
                "f32" => 5,
                "u32" => 5,
                "i32" => 5,
                "char" => 5,
                "u16" => 6,
                "i16" => 6,
                "bool" => 7,
                "u8" => 7,
                "i8" => 7,
                _ => panic!("Not supported: {}", joined),
            }
        }
    }

    fn encode_flatten(&self, offset: usize) -> Tokens<Rust> {
        let name = &self.ident.clone().unwrap().to_string();

        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if joined.starts_with("Option") {
            let p = joined.replace("Option", "");
            let prim = p.as_str();

            let r = quote! {
                if let Some(v) = self.$name {
                    builder.push_slot::<$prim>($offset, v, 0);
                }
            };
            match joined.as_str() {
                "OptionString" => quote! {
                    if let Some(v) = self.$name.clone() {
                        let str_$offset = builder.create_string(v.as_str());
                        builder.push_slot_always($offset, str_$offset);
                    }
                },
                "Optionchar" =>
                // TODO test endianness
                {
                    quote! {
                        if let Some(v) = self.$name {
                            builder.push_slot_always($offset, v as u32);
                        }
                    }
                }
                "Optionbool" => quote! {
                    if let Some(v) = self.$name {
                        builder.push_slot::<bool>($offset, v, false);
                    }
                },
                "Optionf32" => quote! {
                    if let Some(v) = self.$name {
                        builder.push_slot::<f32>($offset, v, 0.0);
                    }
                },
                "Optionf64" => quote! {
                    if let Some(v) = self.$name {
                        builder.push_slot::<f64>($offset, v, 0.0);
                    }
                },
                "Optioni8" => r,
                "Optionu8" => r,
                "Optioni16" => r,
                "Optionu16" => r,
                "Optioni32" => r,
                "Optionu32" => r,
                "Optioni64" => r,
                "Optionu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else if joined.starts_with("Vec") {
            let r = quote! {
                builder.push_slot_always($offset, vec_$offset);
            };
            match joined.as_str() {
                "VecString" => r,
                "Vecchar" => r,
                "Vecbool" => r,
                "Vecf32" => r,
                "Vecf64" => r,
                "Veci8" => r,
                "Vecu8" => r,
                "Veci16" => r,
                "Vecu16" => r,
                "Veci32" => r,
                "Vecu32" => r,
                "Veci64" => r,
                "Vecu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else {
            let r = quote! {
                builder.push_slot::<$(joined.clone())>($offset, self.$name, 0);
            };
            match joined.as_str() {
                "String" => quote! {
                  builder.push_slot_always($offset, str_$offset);
                },
                "char" =>
                // TODO test endianness
                {
                    quote! {
                      builder.push_slot_always($offset, self.$name as u32);
                    }
                }
                "bool" => quote! {
                  builder.push_slot::<bool>($offset, self.$name, false);
                },
                "f32" => quote! {
                  builder.push_slot::<f32>($offset, self.$name, 0.0);
                },
                "f64" => quote! {
                  builder.push_slot::<f64>($offset, self.$name, 0.0);
                },
                "i8" => r,
                "u8" => r,
                "i16" => r,
                "u16" => r,
                "i32" => r,
                "u32" => r,
                "i64" => r,
                "u64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        }
    }

    fn encode_flatten_unnested(&self, offset: usize) -> Tokens<Rust> {
        let wip_offset = &rust::import("flatbuffers", "WIPOffset");
        let name = &self.ident.clone().unwrap().to_string();
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if joined.starts_with("Option") {
            quote!()
        } else if joined.starts_with("Vec") {
            let r = quote! {
                let vec_$offset = builder.create_vector(&self.$name.as_slice());
            };
            match joined.as_str() {
                "VecString" => quote! {
                  let strs_vec_$offset = self.$name.iter()
                  .map(|s|builder.create_string(s.as_str()))
                  .collect::<Vec<$wip_offset<&str>>>();
                  let vec_$offset = builder.create_vector(strs_vec_$offset.as_slice());
                },
                "Vecchar" => quote! {
                    let vec_conversion_$offset: Vec<u32> = self.$name.iter().map(|s|u32::from(*s)).collect();
                    let vec_$offset = builder.create_vector(&vec_conversion_$offset.as_slice());
                },
                "Vecbool" => r,
                "Vecf32" => r,
                "Vecf64" => r,
                "Veci8" => r,
                "Vecu8" => r,
                "Veci16" => r,
                "Vecu16" => r,
                "Veci32" => r,
                "Vecu32" => r,
                "Veci64" => r,
                "Vecu64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        } else {
            let r = quote!();
            match joined.as_str() {
                "String" => quote! {
                    let str_$offset = builder.create_string(self.$name.as_str());
                },
                "char" => r,
                "bool" => r,
                "f32" => r,
                "f64" => r,
                "i8" => r,
                "u8" => r,
                "i16" => r,
                "u16" => r,
                "i32" => r,
                "u32" => r,
                "i64" => r,
                "u64" => r,
                _ => panic!("Not supported: {}", joined),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use darling::FromDeriveInput;

    use super::*;

    #[test]
    fn it_works() {
        let input = syn::parse_str(
            r#"
                #[derive(LeanBufferInternal)]
                struct Entity {
                    t_u64: u64,
                    t_i64: i64,
                    t_u32: u32,
                    t_i32: i32,
                    t_char: char,
                    t_u16: u16,
                    t_i16: i16,
                    t_u8: u8,
                    t_i8: i8,
                    t_bool: bool,
                    t_string: String,
                    t_vec_string: Vec<String>,
                    t_vec_u8: Vec<u8>,
                    t_double: f64,
                    t_float: f32,
                }
            "#,
        )
        .unwrap();
        let receiver = InputReceiver::from_derive_input(&input).unwrap();
        assert!(receiver.data.is_struct());

        // TODO Box or Rc instances, and define a `fn get_fields(&self) -> Vec<Rc<clone>>`
        let fields = receiver
            .data
            .as_ref()
            .take_struct()
            .expect("Enums are not supported (yet)")
            .fields;

        assert_eq!(15, fields.len());
    }
}
