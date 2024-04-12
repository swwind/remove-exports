use std::collections::{HashMap, HashSet};

use swc_ecmascript::{
  ast::{
    Decl, DefaultDecl, ExportSpecifier, Id, Ident, ImportSpecifier, ModuleDecl, ModuleExportName,
    ModuleItem, ObjectPatProp, Pat, Stmt,
  },
  visit::Visit,
};

#[derive(Default, Debug)]
pub struct ImportVisitor {
  decls: HashSet<Id>,
  exports: HashMap<String, HashSet<Id>>,
}

impl ImportVisitor {
  fn register_decl(&mut self, id: Id) {
    self.decls.insert(id);
  }

  fn register_export(&mut self, name: String, id: Id) {
    self
      .exports
      .entry(name)
      .or_insert_with(|| HashSet::new())
      .insert(id);
  }
}

fn find_pat_idents(n: &Pat) -> Vec<Id> {
  match n {
    Pat::Ident(i) => vec![i.to_id()],
    Pat::Array(a) => a
      .elems
      .iter()
      .flatten()
      .flat_map(|e| find_pat_idents(e))
      .collect(),
    Pat::Rest(r) => find_pat_idents(&r.arg),
    Pat::Object(o) => o
      .props
      .iter()
      .flat_map(|i| match i {
        ObjectPatProp::KeyValue(k) => find_pat_idents(&k.value),
        ObjectPatProp::Assign(a) => vec![a.key.id.to_id()],
        ObjectPatProp::Rest(r) => find_pat_idents(&r.arg),
      })
      .collect(),
    Pat::Assign(a) => find_pat_idents(&a.left),

    // ???
    Pat::Invalid(_) => vec![],
    Pat::Expr(_) => vec![],
  }
}

impl Visit for ImportVisitor {
  fn visit_module_item(&mut self, n: &ModuleItem) {
    match n {
      ModuleItem::ModuleDecl(decl) => {
        match decl {
          ModuleDecl::Import(decl) => {
            for specifier in &decl.specifiers {
              match specifier {
                // import { a as b } from "..."
                ImportSpecifier::Named(name) => {
                  self.register_decl(name.local.to_id());
                }
                // import mod from "..."
                ImportSpecifier::Default(def) => {
                  self.register_decl(def.local.to_id());
                }
                // import * as mod from "..."
                ImportSpecifier::Namespace(ns) => {
                  self.register_decl(ns.local.to_id());
                }
              }
            }
          }
          ModuleDecl::ExportDecl(decl) => match &decl.decl {
            // export class foo {}
            Decl::Class(c) => {
              self.register_decl(c.ident.to_id());
              self.register_export(c.ident.to_string(), c.ident.to_id());
            }
            // export function foo {}
            // export function* foo {}
            Decl::Fn(f) => {
              self.register_decl(f.ident.to_id());
              self.register_export(f.ident.to_string(), f.ident.to_id());
            }
            // export const foo = ...
            Decl::Var(v) => {
              v.decls
                .iter()
                .flat_map(|x| find_pat_idents(&x.name))
                .map(|x| Ident::from(x))
                .for_each(|ident| {
                  self.register_decl(ident.to_id());
                  self.register_export(ident.to_string(), ident.to_id());
                });
            }

            // invalid
            Decl::Using(_) => {}
            Decl::TsInterface(_) => {}
            Decl::TsTypeAlias(_) => {}
            Decl::TsEnum(_) => {}
            Decl::TsModule(_) => {}
          },

          ModuleDecl::ExportDefaultDecl(decl) => match &decl.decl {
            // export default class {}
            // export default class foo {}
            DefaultDecl::Class(c) => {
              if let Some(ident) = &c.ident {
                self.register_decl(ident.to_id());
                self
                  .exports
                  .entry("default".to_string())
                  .or_insert_with(|| HashSet::new())
                  .insert(ident.to_id());
              }
            }

            // export default function () {}
            // export default function* () {}
            // export default function foo() {}
            // export default function* foo() {}
            // export default async function () {}
            // export default async function* () {}
            // export default async function foo() {}
            // export default async function* foo() {}
            DefaultDecl::Fn(f) => {
              if let Some(ident) = &f.ident {
                self.register_decl(ident.to_id());
                self
                  .exports
                  .entry("default".to_string())
                  .or_insert_with(|| HashSet::new())
                  .insert(ident.to_id());
              }
            }

            // invalid
            DefaultDecl::TsInterfaceDecl(_) => {}
          },
          // export default foo;
          // do nothing;
          ModuleDecl::ExportDefaultExpr(_) => {}
          // export * from "source";
          // do nothing;
          ModuleDecl::ExportAll(_) => {}

          ModuleDecl::ExportNamed(name) => {
            // export { foo, bar as foo };
            if name.src.is_none() {
              for specifier in &name.specifiers {
                match specifier {
                  ExportSpecifier::Named(name) => {
                    // export { foo as bar }
                    if let Some(exported) = &name.exported {
                      let orig_id = match &name.orig {
                        ModuleExportName::Ident(i) => i.to_id(),
                        ModuleExportName::Str(_) => panic!("invalid code"),
                      };

                      let exported_name = match exported {
                        ModuleExportName::Ident(i) => i.sym.to_string(),
                        ModuleExportName::Str(s) => s.value.to_string(),
                      };

                      self.register_export(exported_name, orig_id);
                    }
                    // export { foo }
                    else {
                      let ident = match &name.orig {
                        ModuleExportName::Ident(i) => i,
                        ModuleExportName::Str(_) => panic!("invalid code"),
                      };

                      self.register_export(ident.to_string(), ident.to_id());
                    }
                  }
                  // invalid
                  ExportSpecifier::Namespace(_) => panic!("invalid code"),
                  ExportSpecifier::Default(_) => panic!("invalid code"),
                }
              }
            }

            // export { foo } from "source";
            // just ignore
          }

          // invalid
          ModuleDecl::TsImportEquals(_) => {}
          ModuleDecl::TsExportAssignment(_) => {}
          ModuleDecl::TsNamespaceExport(_) => {}
        }
      }
      ModuleItem::Stmt(stmt) => {
        if let Stmt::Decl(decl) = stmt {
          match decl {
            Decl::Class(c) => self.register_decl(c.ident.to_id()),
            Decl::Fn(f) => self.register_decl(f.ident.to_id()),
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
