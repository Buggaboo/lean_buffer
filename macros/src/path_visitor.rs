/* Recursively determine the Ident objects, from a syn::Type::Path object */

use syn::{GenericArgument, Ident, PathArguments, Type};

/// This Visitor will be used to traverse a syn::Type::Path object and recursively determine the Ident objects
struct PathVisitor {
    idents: Vec<Ident>,
}

impl PathVisitor {
    fn visit_type(&mut self, i: &Type) {
        if let Type::Path(p) = i {
            for seg in &p.path.segments {
                self.idents.push(seg.ident.clone());

                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in args.args.iter() {
                        if let GenericArgument::Type(a) = arg {
                            self.visit_type(a);
                        }
                    }
                }
            }
        }
    }
}

/// This function would take a syn::Type::Path object as an argument and returns a Vec<Ident>
pub fn get_idents_from_path(path: &Type) -> Vec<Ident> {
    let mut visitor = PathVisitor { idents: Vec::new() };
    visitor.visit_type(path);
    visitor.idents
}

#[cfg(test)]
#[test]
fn recursively_get_idents() {
    for path in vec![
        syn::parse_quote!(std::vec::Vec<String>),
        syn::parse_quote!(vec::Vec<String>),
        syn::parse_quote!(Vec<String>),
    ] {
        let idents = get_idents_from_path(&path);
        assert!(idents.iter().any(|i| i.to_string().contains("Vec")));
        assert!(idents.iter().any(|i| i.to_string().contains("String")));
    }
}
