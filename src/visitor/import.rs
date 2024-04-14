use std::collections::{HashMap, HashSet};

use swc_ecmascript::{
  ast::{
    Decl, DefaultDecl, ExportSpecifier, Id, Ident, ImportSpecifier, ModuleDecl, ModuleExportName,
    ModuleItem, ObjectPatProp, Pat, Stmt,
  },
  visit::{noop_visit_type, Visit},
};

use super::CountVisitor;

#[derive(Default, Debug)]
pub struct ImportVisitor {
  pub decl_refs: HashMap<Id, HashSet<Id>>,
  pub global_refs: HashSet<Id>,
  pub export_decls: HashMap<String, Id>,
  pub export_default_refs: HashSet<Id>,
}

impl ImportVisitor {
  fn insert_decl_refs(&mut self, id: Id, refs: HashSet<Id>) {
    self
      .decl_refs
      .entry(id)
      .or_insert_with(|| HashSet::new())
      .extend(refs)
  }

  fn insert_decls_refs(&mut self, ids: &[Id], refs: &HashSet<Id>) {
    for id in ids {
      self.insert_decl_refs(id.clone(), refs.clone());
    }
  }

  fn insert_global_refs(&mut self, refs: HashSet<Id>) {
    self.global_refs.extend(refs);
  }

  fn insert_export_decl(&mut self, name: String, id: Id) {
    self.export_decls.insert(name, id);
  }

  fn insert_export_decl_ident(&mut self, ident: &Ident) {
    self
      .export_decls
      .insert(ident.sym.to_string(), ident.to_id());
  }

  fn insert_export_default_refs(&mut self, refs: HashSet<Id>) {
    self.export_default_refs.extend(refs);
  }

  fn register_decl(&mut self, id: Id) {
    self.decl_refs.entry(id).or_insert_with(|| HashSet::new());
  }
}

impl ImportVisitor {
  fn find_idents(&mut self, n: &Pat) -> Vec<Id> {
    match n {
      Pat::Ident(i) => vec![i.to_id()],
      Pat::Array(a) => a
        .elems
        .iter()
        .flatten()
        .flat_map(|x| self.find_idents(x))
        .collect(),
      Pat::Rest(r) => self.find_idents(&r.arg),
      Pat::Object(o) => o
        .props
        .iter()
        .flat_map(|x| match x {
          ObjectPatProp::KeyValue(kv) => self.find_idents(&kv.value),
          ObjectPatProp::Assign(ass) => {
            if let Some(value) = &ass.value {
              let refs = CountVisitor::count(value);
              self.insert_decl_refs(ass.key.to_id(), refs);
            }
            return vec![ass.key.to_id()];
          }
          ObjectPatProp::Rest(rest) => self.find_idents(&rest.arg),
        })
        .collect(),
      Pat::Assign(ass) => {
        let ids = self.find_idents(&ass.left);
        let refs = CountVisitor::count(&ass.right);
        self.insert_decls_refs(&ids, &refs);
        ids
      }

      // ignore
      Pat::Invalid(_) => panic!("invalid code"),
      Pat::Expr(_) => panic!("invalid code"),
    }
  }
}

impl Visit for ImportVisitor {
  noop_visit_type!();

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
              let refs = CountVisitor::count(&c.class);
              self.insert_decl_refs(c.ident.to_id(), refs);
              self.insert_export_decl_ident(&c.ident);
            }
            // export function foo {}
            // export function* foo {}
            Decl::Fn(f) => {
              let refs = CountVisitor::count(&f.function);
              self.insert_decl_refs(f.ident.to_id(), refs);
              self.insert_export_decl_ident(&f.ident);
            }
            // export const foo = ...
            Decl::Var(v) => {
              for decl in &v.decls {
                let ids = self.find_idents(&decl.name);

                let refs = match &decl.init {
                  Some(init) => CountVisitor::count(init),
                  None => HashSet::new(),
                };
                self.insert_decls_refs(&ids, &refs);

                for id in ids {
                  let ident = Ident::from(id);
                  self.insert_export_decl_ident(&ident);
                }
              }
            }

            // invalid
            Decl::Using(_) => panic!("invalid code"),
            Decl::TsInterface(_) => panic!("invalid code"),
            Decl::TsTypeAlias(_) => panic!("invalid code"),
            Decl::TsEnum(_) => panic!("invalid code"),
            Decl::TsModule(_) => panic!("invalid code"),
          },

          ModuleDecl::ExportDefaultDecl(decl) => match &decl.decl {
            // export default class {}
            // export default class foo {}
            DefaultDecl::Class(c) => {
              let refs = CountVisitor::count(&c.class);
              if let Some(ident) = &c.ident {
                self.register_decl(ident.to_id());
              }
              self.insert_export_default_refs(refs);
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
              let refs = CountVisitor::count(&f.function);
              if let Some(ident) = &f.ident {
                self.register_decl(ident.to_id());
              }
              self.insert_export_default_refs(refs);
            }

            // invalid
            DefaultDecl::TsInterfaceDecl(_) => panic!("invalid code"),
          },

          // export default foo;
          // do nothing;
          ModuleDecl::ExportDefaultExpr(expr) => {
            let refs = CountVisitor::count(&expr.expr);
            self.insert_export_default_refs(refs);
          }

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

                      self.insert_export_decl(exported_name, orig_id);
                    }
                    // export { foo }
                    else {
                      let ident = match &name.orig {
                        ModuleExportName::Ident(i) => i,
                        ModuleExportName::Str(_) => panic!("invalid code"),
                      };

                      self.insert_export_decl_ident(&ident);
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

      ModuleItem::Stmt(stmt) => match stmt {
        Stmt::Decl(decl) => {
          match decl {
            Decl::Class(c) => {
              let refs = CountVisitor::count(&c.class);
              self.insert_decl_refs(c.ident.to_id(), refs);
            }
            Decl::Fn(f) => {
              let refs = CountVisitor::count(&f.function);
              self.insert_decl_refs(f.ident.to_id(), refs);
            }
            Decl::Var(v) => {
              for decl in &v.decls {
                let ids = self.find_idents(&decl.name);
                let refs = match &decl.init {
                  Some(init) => CountVisitor::count(init),
                  None => HashSet::new(),
                };
                self.insert_decls_refs(&ids, &refs);
              }
            }

            // invalid
            Decl::Using(_) => panic!("invalid code"),
            Decl::TsInterface(_) => panic!("invalid code"),
            Decl::TsTypeAlias(_) => panic!("invalid code"),
            Decl::TsEnum(_) => panic!("invalid code"),
            Decl::TsModule(_) => panic!("invalid code"),
          }
        }
        Stmt::Block(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Empty(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Debugger(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::With(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Return(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Labeled(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Break(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Continue(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::If(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Switch(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Throw(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Try(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::While(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::DoWhile(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::For(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::ForIn(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::ForOf(x) => self.insert_global_refs(CountVisitor::count(x)),
        Stmt::Expr(x) => self.insert_global_refs(CountVisitor::count(x)),
      },
    }
  }
}
