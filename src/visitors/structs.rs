use cdd::{Variable, VariableType};
use std::collections::HashMap;
use syn::visit::Visit;
use syn::{Fields, Item, Type};

pub fn extract_structures_from_code(
    code: &str,
) -> Result<HashMap<String, Vec<Variable>>, failure::Error> {
    let mut visitor = StructVisitor::new();
    let syntax = syn::parse_file(&code)?;
    syn::visit::visit_file(&mut visitor, &syntax);

    Ok(visitor.structs)
}

pub(crate) fn syn_type_to_cdd_type(ty: &str) -> VariableType {
    match ty {
        "String" => VariableType::StringType,
        "u32" | "u64" | "i32" | "i64" => VariableType::IntType,
        "f64" => VariableType::FloatType,
        "bool" => VariableType::BoolType,
        "Vec" => VariableType::ArrayType(Box::new(VariableType::StringType)), // fix
        _ => VariableType::ComplexType(ty.to_string()),                       // fix
    }
}

fn type_is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(typepath) => typepath.path.segments.iter().next().unwrap().ident == "Option",
        _ => false,
    }
}

fn extract_variables(fields: &Fields) -> Vec<cdd::Variable> {
    fn extract_ident_from_type(ty: &Type) -> String {
        match ty {
            Type::Path(typepath) => typepath
                .path
                .segments
                .iter()
                .next()
                .unwrap()
                .ident
                .to_string(),
            _ => panic!("oops, extracting ident from type failed"),
        }
    }

    fn extract_name_from_field(field: &syn::Field) -> String {
        match &field.ident {
            Some(i) => i.to_string(),
            None => panic!("oops, sent an empty var"),
        }

        // format!("{:?}", field)
        // match ty {
        //     Type::Path(typepath) => typepath.path.segments.iter().next().unwrap().ident.to_string(),
        //     _ => panic!("oops, extracting ident from type failed"),
        // }
    }

    let mut vars = Vec::new();

    for field in fields.iter() {
        let name = extract_name_from_field(&field);
        let ty = &field.ty;
        let optional = type_is_option(&ty);
        let variable_type = if type_is_option(&ty) {
            // extract_ident_from_type(extract_type_from_option(&ty).unwrap())
            "None".into()
        } else {
            extract_ident_from_type(&ty)
        };
        let variable_type = syn_type_to_cdd_type(&variable_type);

        vars.push(cdd::Variable {
            name,
            optional,
            value: None,
            variable_type,
        });
    }
    vars
}

pub(crate) struct StructVisitor {
    pub structs: HashMap<String, Vec<cdd::Variable>>,
}

impl StructVisitor {
    pub(crate) fn new() -> Self {
        Self {
            structs: HashMap::new(),
        }
    }
}

impl<'ast> Visit<'ast> for StructVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        match i {
            Item::Struct(s) => {
                let struct_name = format!("{}", s.ident);
                let variables = extract_variables(&s.fields);
                self.structs.insert(struct_name, variables);
            }
            _ => (),
        };
    }
}

#[test]
fn test_expr_parse() {
    let code = r#"
        // a comment
        struct StructOne { example: String }
        struct StructTwo;
    "#;
    let mut visitor = StructVisitor::new();
    let syntax = syn::parse_file(&code).unwrap();
    syn::visit::visit_file(&mut visitor, &syntax);

    assert!(visitor.structs.contains_key("StructOne"));
    assert!(visitor.structs.contains_key("StructTwo"));
    assert_eq!(visitor.structs.len(), 2);
    assert_eq!(visitor.structs["StructOne"].len(), 1);
    assert_eq!(visitor.structs["StructTwo"].len(), 0);
}

#[test]
fn test_var_parse() {
    let code = r#"
        struct Dog {
            name: String,
            age: f64,
        }
    "#;
    let mut visitor = StructVisitor::new();
    let syntax = syn::parse_file(&code).unwrap();
    syn::visit::visit_file(&mut visitor, &syntax);

    assert!(visitor.structs.contains_key("Dog"));
    assert_eq!(visitor.structs.len(), 1);
    assert_eq!(visitor.structs["Dog"].len(), 2);

    let dog_name: cdd::Variable = visitor.structs["Dog"]
        .iter()
        .find(|v| v.name == "name")
        .unwrap()
        .clone();

    assert_eq!(dog_name.variable_type, cdd::VariableType::StringType);

    let dog_age: cdd::Variable = visitor.structs["Dog"]
        .iter()
        .find(|&v| v.name == "age")
        .unwrap()
        .clone();

    assert_eq!(dog_age.variable_type, cdd::VariableType::FloatType);
}

// fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
//     use syn::punctuated::Pair;
//     use syn::token::Colon2;
//     use syn::{GenericArgument, Path, PathArguments, PathSegment};

//     fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
//         match *ty {
//             syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
//             _ => None,
//         }
//     }

//     // TODO store (with lazy static) the vec of string
//     // TODO maybe optimization, reverse the order of segments
//     fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
//         let idents_of_path = path
//             .segments
//             .iter()
//             .into_iter()
//             .fold(String::new(), |mut acc, v| {
//                 acc.push_str(&v.ident.to_string());
//                 acc.push('|');
//                 acc
//             });
//         vec!["Option|", "std|option|Option|", "core|option|Option|"]
//             .into_iter()
//             .find(|s| &idents_of_path == *s)
//             .and_then(|_| path.segments.last())
//     }

//     extract_type_path(ty)
//         .and_then(extract_option_segment)
//         // .and_then(|pair_path_segment| {
//         //     let type_params = &pair_path_segment.into_value().arguments;
//         //     // It should have only on angle-bracketed param ("<String>"):
//         //     match *type_params {
//         //         PathArguments::AngleBracketed(ref params) => params.args.first(),
//         //         _ => None,
//         //     }
//         // })
//         .and_then(|generic_arg| match generic_arg {
//             GenericArgument::Type(ref ty) => Some(ty),
//             _ => None,
//         })
// }
