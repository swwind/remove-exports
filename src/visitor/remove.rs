use std::collections::HashSet;

use swc_ecmascript::{
  ast::{
    Decl, ExportSpecifier, Ident, ModuleDecl, ModuleExportName, ModuleItem, ObjectPatProp, Pat,
  },
  visit::VisitMut,
};

pub struct RemoveVisitor {
  pub removes: HashSet<String>,
}

impl RemoveVisitor {
  fn should_remove_ident(&self, ident: &Ident) -> bool {
    self.removes.contains(&ident.sym.to_string())
  }

  fn should_remove_module_export(&self, n: &ModuleExportName) -> bool {
    self.removes.contains(&match n {
      ModuleExportName::Ident(ident) => ident.sym.to_string(),
      ModuleExportName::Str(s) => s.value.to_string(),
    })
  }

  fn should_remove_pat(&mut self, n: &mut Pat) -> bool {
    match n {
      // foo
      Pat::Ident(i) => self.should_remove_ident(&i.id),
      // [ foo, bar ]
      Pat::Array(a) => {
        a.elems.iter_mut().for_each(|x| {
          if x.as_mut().is_some_and(|p| self.should_remove_pat(p)) {
            *x = None;
          }
        });
        a.elems.iter().all(|x| x.is_none())
      }
      // { foo, bar }
      Pat::Object(o) => {
        o.props.retain_mut(|i| match i {
          // { key: value }
          ObjectPatProp::KeyValue(kv) => !self.should_remove_pat(&mut kv.value),
          // { foo = 233 }
          ObjectPatProp::Assign(a) => !self.should_remove_ident(&a.key.id),
          // { ...rest }
          ObjectPatProp::Rest(rs) => !self.should_remove_pat(&mut rs.arg),
        });
        o.props.is_empty()
      }
      // [ ...bar ]
      Pat::Rest(rs) => self.should_remove_pat(&mut rs.arg),
      // [ foo = ... ]
      Pat::Assign(a) => self.should_remove_pat(&mut a.left),
      // ???
      Pat::Expr(_) => false,
      Pat::Invalid(_) => false,
    }
  }

  fn should_remove_module_decl(&mut self, n: &mut ModuleDecl) -> bool {
    match n {
      ModuleDecl::ExportDecl(decl) => {
        match &mut decl.decl {
          // export class foo { }
          Decl::Class(c) => self.removes.contains(&c.ident.sym.to_string()),

          // export function foo() { }
          Decl::Fn(f) => self.removes.contains(&f.ident.sym.to_string()),

          // export const foo = ...
          // export let foo = ...
          // export var foo = ...
          Decl::Var(v) => {
            v.decls.retain_mut(|f| !self.should_remove_pat(&mut f.name));
            v.decls.is_empty()
          }

          // export enum Foo { }
          Decl::TsEnum(f) => self.removes.contains(&f.id.sym.to_string()),

          // export using foo = ...
          Decl::Using(_) => unreachable!(),
          // export interface Foo {}
          Decl::TsInterface(_) => unreachable!(),
          // export type Foo = ...
          Decl::TsTypeAlias(_) => unreachable!(),
          // export declare module "xxx" { }
          Decl::TsModule(_) => unreachable!(),
        }
      }

      ModuleDecl::ExportNamed(named) => {
        named.specifiers.retain(|exp| match exp {
          // export * as foo from "source"
          ExportSpecifier::Namespace(namespace) => {
            !self.should_remove_module_export(&namespace.name)
          }
          // export { name, foo as bar };
          // export { name, foo as bar } from "source";
          ExportSpecifier::Named(named) => {
            !self.should_remove_module_export(match &named.exported {
              Some(exported) => exported,
              None => &named.orig,
            })
          }
          // export v from "source";
          ExportSpecifier::Default(_) => unreachable!(),
        });
        named.specifiers.is_empty()
      }

      // export default class {}
      // export default function () {}
      ModuleDecl::ExportDefaultDecl(_) => self.removes.contains("default"),
      // export default <expr>;
      ModuleDecl::ExportDefaultExpr(_) => self.removes.contains("default"),

      // import "source";
      // import { ... } from "source";
      ModuleDecl::Import(_) => false,
      // export * from "source";
      ModuleDecl::ExportAll(_) => false,
      // import rust = go;
      ModuleDecl::TsImportEquals(_) => false,
      // export = <expr>;
      ModuleDecl::TsExportAssignment(_) => false,
      // export as namespace Rust;
      ModuleDecl::TsNamespaceExport(_) => false,
    }
  }
}

impl VisitMut for RemoveVisitor {
  fn visit_mut_module_items(&mut self, n: &mut Vec<ModuleItem>) {
    n.retain_mut(|x| match x {
      ModuleItem::ModuleDecl(decl) => !self.should_remove_module_decl(decl),
      ModuleItem::Stmt(_) => true,
    });
  }
}
