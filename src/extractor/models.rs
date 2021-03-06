use cdd::*;

pub fn extract_from_ast(syntax: &syn::File) -> Result<Vec<Model>, failure::Error> {
    let mut visitor = crate::visitors::StructVisitor::new();
    syn::visit::visit_file(&mut visitor, &syntax);

    Ok(visitor
        .structs
        .into_iter()
        .map(|(name, vars)| Model {
            name,
            vars: vars.into_iter().map(|v| Box::new(v)).collect(),
        })
        .collect())
}
