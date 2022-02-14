use crate::{
    mutation::{Mutation, MutationOp},
    source::SourceFile,
};
use syn::visit::Visit;

/// `syn` visitor that recursively traverses the syntax tree, accumulating
/// places that could be mutated.
pub struct Visitor<'sf> {
    /// All the mutations generated by visiting the file.
    pub mutations: Vec<Mutation>,

    /// The file being visited.
    source_file: &'sf SourceFile,
}

impl<'sf> Visitor<'sf> {
    pub fn new(source_file: &'sf SourceFile) -> Self {
        Self {
            source_file,
            mutations: Vec::new(),
        }
    }

    fn ops_for_visibility(&self, vis: &syn::Visibility) -> Vec<Mutation> {
        match vis {
            syn::Visibility::Public(pv) => {
                let ops = &[
                    MutationOp::ToVisCrate,
                    MutationOp::ToVisInherited,
                    MutationOp::ToVisSelf,
                    MutationOp::ToVisSuper,
                ];
                let span = pv.pub_token.span.into();
                ops.iter()
                    .map(|op| Mutation::new(self.source_file.clone(), *op, span))
                    .collect()
            }
            syn::Visibility::Crate(cv) => {
                let ops = &[
                    MutationOp::ToVisSelf,
                    MutationOp::ToVisSuper,
                    MutationOp::ToVisInherited,
                ];
                let span = cv.crate_token.span.into();
                ops.iter()
                    .map(|op| Mutation::new(self.source_file.clone(), *op, span))
                    .collect()
            }
            syn::Visibility::Restricted(..) => vec![], // TODO support
            syn::Visibility::Inherited => vec![],
        }
    }
}

impl<'ast, 'sf> Visit<'ast> for Visitor<'sf> {
    /// Visit `struct`
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.mutations.extend(self.ops_for_visibility(&node.vis));
        syn::visit::visit_fields(self, &node.fields);
    }

    /// Visit `fn`
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.mutations.extend(self.ops_for_visibility(&node.vis));
    }

    /// Visit `enum`
    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.mutations.extend(self.ops_for_visibility(&node.vis));
        for variant in node.variants.iter() {
            syn::visit::visit_variant(self, &variant);
        }
    }

    /// Visit `enum` variants
    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        syn::visit::visit_fields(self, &node.fields);
    }

    /// Visit fields of structs, enums etc
    fn visit_fields(&mut self, node: &'ast syn::Fields) {
        if let syn::Fields::Named(named) = node {
            for field in named.named.iter() {
                syn::visit::visit_field(self, field);
            }
        }
    }

    /// Visit field, wherever it is
    fn visit_field(&mut self, node: &'ast syn::Field) {
        self.mutations.extend(self.ops_for_visibility(&node.vis))
    }
}
