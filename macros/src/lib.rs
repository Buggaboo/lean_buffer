#![allow(dead_code)]

mod path_visitor;

use std::{str::FromStr, env, path::Path, fs};

use darling::{ast, FromDeriveInput, FromField};
use genco::{prelude::{Rust, rust}, Tokens, quote};
use proc_macro::TokenStream;
use syn::DeriveInput;

use genco::fmt;

fn tokens_to_string(tokens: &Tokens<Rust>) -> Vec<u8> {
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

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(lean_buffer_internal), supports(struct_any))]
struct InputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), FieldReceiver>,
}

impl InputReceiver {
    fn write(&mut self) {
        if let Some(out_dir) = env::var_os("OUT_DIR") {
            let dest_path =
                Path::new(&out_dir).join(format!("{}_lb_gen.rs", self.ident.to_string().clone()));
            let code = self.generate_code();
            if let Err(error) = fs::write(&dest_path, code.as_str()) {
                panic!(
                    "There is a problem writing the generated rust code: {:?}",
                    error
                );
            }    
        } else {
            panic!("Missing OUT_DIR environment variable, add a `build.rs` with at least an empty `fn main` to the root of your project");
        }
    }

    fn generate_code(&self) -> String {
        let tokens = &mut rust::Tokens::new();

        tokens.append(self.generate_factory());
        tokens.append(self.generate_table_adapter());

        let vector = tokens_to_string(tokens);

        let utf = match std::str::from_utf8(vector.as_slice()) {
            Ok(utf) => utf,
            Err(error) => panic!(
                "There is a problem with converting bytes to utf8: {}",
                error
            ),
        };

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

    fn generate_factory(&self) -> Tokens<Rust> {
        // TODO Box or Rc instances, and define a `fn get_fields(&self) -> Vec<Rc<clone>>`
        let fields = self.data
            .as_ref()
            .take_struct()
            .expect("Enums are not supported (yet)")
            .fields;

        let fb_table = &rust::import("flatbuffers", "Table");
        let factory = &rust::import("lean_buffer::traits", "Factory");
        let factory_ext = &rust::import("lean_buffer::traits", "FactoryExt");
        let entity = &rust::import("crate", &self.ident.to_string());

        let destructured_props = fields
            .iter()
            .map(|p| p.as_struct_property_default());
        let assigned_props = fields
            .iter()
            .enumerate()
            .map(|p| p.1.as_assigned_property(p.0 * 2 + 4));

        quote! {
          impl $factory_ext<$entity> for $factory<$entity> {
            fn make(&self, table: &mut $fb_table) -> $entity {
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
        let entity = &rust::import("crate", &self.ident.to_string());
        let bridge_trait = &rust::import("lean_buffer::traits", "AdapterExt");
        let flatbuffer_builder = &rust::import("flatbuffers", "FlatBufferBuilder");

        // TODO Box or Rc instances, and define a `fn get_fields(&self) -> Vec<Rc<clone>>`
        let fields = self.data
            .as_ref()
            .take_struct()
            .expect("Enums are not supported (yet)")
            .fields;

        let unnested_props: Vec<Tokens<Rust>> = fields
            .iter()
            .enumerate()
            .map(|(i, p)| p.encode_to_fb_unnested(i * 2 + 4))
            .collect();

        let mut props_unsorted: Vec<(usize, Tokens<Rust>)> = fields
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    p.to_sorting_priority(),
                    p.encode_to_fb(i * 2 + 4),
                )
            })
            .collect();

        props_unsorted.sort_by(|a, b| a.0.cmp(&b.0));
        let props: Vec<Tokens<Rust>> = props_unsorted.iter().map(|t| t.1.clone()).collect();

        quote! {
          impl $bridge_trait for $entity {
            fn to_fb(&self, builder: &mut $flatbuffer_builder) {
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
struct FieldReceiver {
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
        if Self::allowed_types(joined.as_str()) || joined.eq("VecString") {
            let prim = joined.replace("Vec", "");
            return quote! {
                $name: Vec::<$prim>::new()
            };
        }

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
            // rest of the integer types
            _ => quote! {
                $name: 0
            },
        }
    }

    fn allowed_types(joined: &str) -> bool {
        match joined.len() {
            5 => {
                matches!(joined, "Vecu8" | "Veci8")
            }
            6 => {
                (joined.starts_with("Vecu") || joined.starts_with("Veci")) &&
                ["16", "32", "64"].iter().any(|d| joined.ends_with(d))
                || matches!(joined, "Vecf32" | "Vecf64")
            },
            7 => {
                matches!(joined, "Vecbool" | "Vecchar")    
            },
            _ => false
        }
    }

    fn as_assigned_property(&self, offset: usize) -> Tokens<Rust> {
        let fuo = &rust::import("flatbuffers", "ForwardsUOffset");
        let fvec = &rust::import("flatbuffers", "Vector");

        let name = &self.ident.clone().unwrap().to_string();
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if Self::allowed_types(joined.as_str()) {
            let p = joined.replace("Vec", "");
            let prim = p.as_str();

            if joined.ends_with("i8") {
                return quote! {
                    let fb_$name = table.get::<$fuo<$fvec<$prim>>>($offset, None);
                    if let Some(vb) = fb_$name {
                        let vec_u8 = vb.bytes().to_vec();
                        let slice_u8 = vec_u8.as_slice();
                        let slice_i8 = unsafe { &*(slice_u8 as *const _  as *const [i8]) };
                        *$name = slice_i8.to_vec();
                    }
                };
            }

            if joined.ends_with("u8") {
                return quote! {
                    let fb_$name = table.get::<$fuo<$fvec<$prim>>>($offset, None);
                    if let Some(vb) = fb_$name {
                        *$name = vb.bytes().to_vec();
                    }
                };
            }

            if joined.ends_with("char") {
                return quote! {
                    let fb_$name = table.get::<$fuo<$fvec<u32>>>($offset, None);
                    if let Some(c) = fb_$name {
                        *$name = c.iter().filter_map(|s| char::from_u32(s)).collect();
                    }
                };
            }

            return quote! {
                let fb_$name = table.get::<$fuo<$fvec<$prim>>>($offset, None);
                if let Some(v) = fb_$name {
                    *$name = v.iter().map(|s|s).collect();
                }
            };
        }

        match joined.as_str() {
            "VecString" => quote! {
                let fb_vec_$name = table.get::<$fuo<$fvec<$fuo<&str>>>>($offset, None);
                if let Some(sv) = fb_vec_$name {
                    *$name = sv.iter().map(|s|s.to_string()).collect();
                }
            },
            "String" => quote! {
                if let Some(s) = table.get::<$fuo<&str>>($offset, None) {
                    *$name = s.to_string();
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
            // rest of the integer types
            _ => {
                quote! {
                    *$name = table.get::<$joined>($offset, Some(0)).unwrap();
                }
            }
        }
    }

    fn to_sorting_priority(&self) -> usize {
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if Self::allowed_types(joined.as_str()) {
            return 3;
        }

        // naive packing
        match joined.as_str() {
            "f64" => 1,
            "u64" => 1,
            "i64" => 1,
            "VecString" => 2,
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
            _ => 8,
        }
    }

    fn encode_to_fb(&self, offset: usize) -> Tokens<Rust> {
        let name = &self.ident.clone().unwrap().to_string();
        
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();

        if Self::allowed_types(joined.as_str()) {
            return quote! {
                builder.push_slot_always($offset, vec_$offset);
            };
        }

        match joined.as_str() {
            "VecString" => {
                quote! {
                  builder.push_slot_always($offset, vec_$offset);
                }
            },
            "String" => {
                quote! {
                  builder.push_slot_always($offset, str_$offset);
                }
            },
            "char" => {
                // TODO test endianness
                quote! {
                  builder.push_slot_always($offset, self.$name as u32);
                }
            },
            "bool" => {
                quote! {
                  builder.push_slot::<bool>($offset, self.$name, false);
                }
            },
            "f32" => {
                quote! {
                  builder.push_slot::<f32>($offset, self.$name, 0.0);
                }
            },
            "f64" => {
                quote! {
                  builder.push_slot::<f64>($offset, self.$name, 0.0);
                }
            },
            // rest of the primitives
            _ => {
                quote! {
                  builder.push_slot::<$joined>($offset, self.$name, 0);
                }
            }
        }
    }
    
    fn encode_to_fb_unnested(&self, offset: usize) -> Tokens<Rust> {
        let wip_offset = &rust::import("flatbuffers", "WIPOffset");
        let name = &self.ident.clone().unwrap().to_string();
        let ty = path_visitor::get_idents_from_path(&self.ty);
        let joined = ty.iter().map(|i| i.to_string()).collect::<String>();
    
        if Self::allowed_types(joined.as_str()) {
            if joined.ends_with("char") {
                return quote! {
                    let vec_conversion_$offset: Vec<u32> = self.$name.iter().map(|s|u32::from(*s)).collect();
                    let vec_$offset = builder.create_vector(&vec_conversion_$offset.as_slice());
                };
            }

            return quote! {
                let vec_$offset = builder.create_vector(&self.$name.as_slice());
            };
        }

        match joined.as_str() {
            "VecString" => {
                quote! {
                  let strs_vec_$offset = self.$name.iter()
                  .map(|s|builder.create_string(s.as_str()))
                  .collect::<Vec<$wip_offset<&str>>>();
                  let vec_$offset = builder.create_vector(strs_vec_$offset.as_slice());
                }
            },
            "String" => {
                quote! {
                  let str_$offset = builder.create_string(self.$name.as_str());
                }
            },
            _ => quote!(),
        }
    }
}

#[proc_macro_derive(LeanBufferWrite)]
pub fn derive_fb_code_then_write(input: TokenStream) -> TokenStream {
    let mut out = TokenStream::new();
    // yes, nasty hack, to wrap code generation
    out.extend(TokenStream::from_str("#[derive(LeanBufferInternal)]"));
    out.extend(input.clone());
    let parsed = syn::parse::<DeriveInput>(out).expect("crash");
    let mut receiver = InputReceiver::from_derive_input(&parsed).expect("crash");
    receiver.write();
    TokenStream::new()
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
        ).unwrap();
        let receiver = InputReceiver::from_derive_input(&input).unwrap();
        assert!(receiver.data.is_struct());

        // TODO Box or Rc instances, and define a `fn get_fields(&self) -> Vec<Rc<clone>>`
        let fields = receiver.data
            .as_ref()
            .take_struct()
            .expect("Enums are not supported (yet)")
            .fields;

        assert_eq!(15, fields.len());
    }
}