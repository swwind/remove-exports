use swc_ecmascript::{
  ast::{Decl, Id, ImportSpecifier, ModuleDecl, ModuleItem, ObjectPatProp, Pat, Stmt},
  visit::Visit,
};

#[derive(Default, Debug)]
pub struct ImportVisitor {
  imports: Vec<Id>,
  defs: Vec<Id>,
}

impl Visit for ImportVisitor {
  fn visit_pat(&mut self, n: &Pat) {
    match n {
      Pat::Ident(i) => self.defs.push(i.to_id()),
      Pat::Array(a) => a.elems.iter().flatten().for_each(|e| self.visit_pat(e)),
      Pat::Rest(r) => self.visit_pat(&r.arg),
      Pat::Object(o) => {
        for i in &o.props {
          match i {
            ObjectPatProp::KeyValue(k) => self.visit_pat(&k.value),
            ObjectPatProp::Assign(a) => self.defs.push(a.key.id.to_id()),
            ObjectPatProp::Rest(r) => self.visit_pat(&r.arg),
          }
        }
      }
      Pat::Assign(a) => self.visit_pat(&a.left),
      // ???
      Pat::Invalid(_) => {}
      Pat::Expr(_) => {}
    }
  }

  fn visit_module_item(&mut self, n: &ModuleItem) {
    match n {
      ModuleItem::ModuleDecl(decl) => {
        if let ModuleDecl::Import(decl) = decl {
          for specifier in &decl.specifiers {
            match specifier {
              // import { a as b } from "..."
              ImportSpecifier::Named(name) => self.imports.push(name.local.to_id()),
              // import mod from "..."
              ImportSpecifier::Default(def) => self.imports.push(def.local.to_id()),
              // import * as mod from "..."
              ImportSpecifier::Namespace(ns) => self.imports.push(ns.local.to_id()),
            }
          }
        }
      }
      ModuleItem::Stmt(stmt) => {
        if let Stmt::Decl(decl) = stmt {
          match decl {
            Decl::Class(c) => self.defs.push(c.ident.to_id()),
            Decl::Fn(f) => self.defs.push(f.ident.to_id()),
            Decl::Var(v) => {
              for d in &v.decls {
                self.visit_pat(&d.name)
              }
            }
            _ => {}
          }
        }
      }
    }
  }
}
