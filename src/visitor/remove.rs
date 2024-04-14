use std::{
  collections::{HashMap, HashSet, VecDeque},
  fmt::Debug,
  hash::Hash,
};

use swc_ecmascript::{
  ast::{
    Decl, ExportSpecifier, Id, Ident, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem,
    ObjectPatProp, Pat, Stmt,
  },
  visit::{noop_visit_mut_type, VisitMut},
};

use super::ImportVisitor;

#[derive(Debug)]
pub struct RemoveVisitor {
  pub names: HashSet<String>,
  pub ids: HashSet<Id>,
}

impl RemoveVisitor {
  fn should_remove_ident(&self, ident: &Ident) -> bool {
    self.ids.contains(&ident.to_id())
  }

  fn should_remove_module_export(&self, n: &ModuleExportName) -> bool {
    self.names.contains(&match n {
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
      // [ foo = 233 ]
      Pat::Assign(a) => self.should_remove_pat(&mut a.left),
      // ???
      Pat::Expr(_) => panic!("invalid code"),
      Pat::Invalid(_) => panic!("invalid code"),
    }
  }

  fn should_remove_module_decl(&mut self, n: &mut ModuleDecl) -> bool {
    match n {
      ModuleDecl::ExportDecl(decl) => {
        match &mut decl.decl {
          // export class foo { }
          Decl::Class(c) => self.should_remove_ident(&c.ident),

          // export function foo() { }
          Decl::Fn(f) => self.should_remove_ident(&f.ident),

          // export const foo = ...
          // export let foo = ...
          // export var foo = ...
          Decl::Var(v) => {
            v.decls
              .retain_mut(|decl| !self.should_remove_pat(&mut decl.name));
            v.decls.is_empty()
          }

          // export enum Foo { }
          Decl::TsEnum(_) => panic!("invalid code"),
          // export using foo = ...
          Decl::Using(_) => panic!("invalid code"),
          // export interface Foo {}
          Decl::TsInterface(_) => panic!("invalid code"),
          // export type Foo = ...
          Decl::TsTypeAlias(_) => panic!("invalid code"),
          // export declare module "xxx" { }
          Decl::TsModule(_) => panic!("invalid code"),
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
          ExportSpecifier::Default(_) => panic!("invalid code"),
        });
        named.specifiers.is_empty()
      }

      // export default class {}
      // export default function () {}
      ModuleDecl::ExportDefaultDecl(_) => self.names.contains("default"),
      // export default <expr>;
      ModuleDecl::ExportDefaultExpr(_) => self.names.contains("default"),

      // import "source";
      // import { ... } from "source";
      ModuleDecl::Import(import) => {
        let old = import.specifiers.len();
        import.specifiers.retain(|x| match x {
          // import { foo, foo as bar } from "source";
          ImportSpecifier::Named(name) => !self.should_remove_ident(&name.local),
          // import foo from "source";
          ImportSpecifier::Default(def) => !self.should_remove_ident(&def.local),
          // import * as foo from "source";
          ImportSpecifier::Namespace(ns) => !self.should_remove_ident(&ns.local),
        });
        let now = import.specifiers.len();
        now != old && now == 0
      }

      // export * from "source";
      ModuleDecl::ExportAll(_) => false,

      // import rust = go;
      ModuleDecl::TsImportEquals(_) => panic!("invalid code"),
      // export = <expr>;
      ModuleDecl::TsExportAssignment(_) => panic!("invalid code"),
      // export as namespace Rust;
      ModuleDecl::TsNamespaceExport(_) => panic!("invalid code"),
    }
  }
}

impl VisitMut for RemoveVisitor {
  noop_visit_mut_type!();

  fn visit_mut_module_items(&mut self, n: &mut Vec<ModuleItem>) {
    n.retain_mut(|x| match x {
      ModuleItem::ModuleDecl(decl) => !self.should_remove_module_decl(decl),
      ModuleItem::Stmt(stmt) => match stmt {
        Stmt::Decl(decl) => match decl {
          Decl::Class(c) => !self.should_remove_ident(&c.ident),
          Decl::Fn(f) => !self.should_remove_ident(&f.ident),
          Decl::Var(v) => {
            v.decls
              .retain_mut(|decl| !self.should_remove_pat(&mut decl.name));
            !v.decls.is_empty()
          }

          Decl::Using(_) => panic!("invalid code"),
          Decl::TsInterface(_) => panic!("invalid code"),
          Decl::TsTypeAlias(_) => panic!("invalid code"),
          Decl::TsEnum(_) => panic!("invalid code"),
          Decl::TsModule(_) => panic!("invalid code"),
        },
        Stmt::Block(_) => true,
        Stmt::Empty(_) => true,
        Stmt::Debugger(_) => true,
        Stmt::With(_) => true,
        Stmt::Return(_) => true,
        Stmt::Labeled(_) => true,
        Stmt::Break(_) => true,
        Stmt::Continue(_) => true,
        Stmt::If(_) => true,
        Stmt::Switch(_) => true,
        Stmt::Throw(_) => true,
        Stmt::Try(_) => true,
        Stmt::While(_) => true,
        Stmt::DoWhile(_) => true,
        Stmt::For(_) => true,
        Stmt::ForIn(_) => true,
        Stmt::ForOf(_) => true,
        Stmt::Expr(_) => true,
      },
    });
  }
}

struct RefCounter<K> {
  map: HashMap<K, u32>,
  done: HashSet<K>,
}

impl<K> RefCounter<K>
where
  K: Hash + Eq + Clone + Debug,
{
  fn from_keys(keys: Vec<K>) -> Self {
    let mut map = HashMap::new();
    for key in keys {
      map.entry(key).or_insert(0);
    }
    let done = HashSet::new();
    Self { map, done }
  }

  fn count(&mut self, key: &K) {
    println!("count: {:?}", key);
    self.map.entry(key.clone()).and_modify(|x| *x += 1);
  }

  fn discount(&mut self, key: &K, f: impl FnOnce(&K)) {
    println!("discount: {:?}", key);
    if let Some(x) = self.map.get_mut(key) {
      if *x == 0 {
        // maybe this is force-removed
        return;
      }
      *x -= 1;
      if *x == 0 {
        self.mark(key);
        f(key);
      }
    }
  }

  fn mark(&mut self, key: &K) {
    self.done.insert(key.clone());
  }
}

impl RemoveVisitor {
  pub fn new(imports: ImportVisitor, removes: Vec<String>) -> Self {
    // analyze every keys refs counts
    let mut ref_counts = RefCounter::from_keys(imports.decl_refs.keys().cloned().collect());

    for (key, values) in &imports.decl_refs {
      for value in values {
        if key != value {
          ref_counts.count(value);
        }
      }
    }
    for value in &imports.global_refs {
      ref_counts.count(value);
    }
    for value in &imports.export_default_refs {
      ref_counts.count(value);
    }

    // repeatly mark decls as should remove
    let mut queue = VecDeque::<Id>::new();
    for (name, id) in &imports.export_decls {
      if removes.contains(name) {
        if let Some((k, v)) = imports.decl_refs.get_key_value(id) {
          ref_counts.mark(k);
          for id in v {
            if id != k {
              ref_counts.discount(id, |id| queue.push_back(id.clone()));
            }
          }
        }
      }
    }
    if removes.contains(&"default".to_string()) {
      for id in &imports.export_default_refs {
        ref_counts.discount(id, |id| queue.push_back(id.clone()));
      }
    }

    while let Some(id) = queue.pop_front() {
      if let Some(set) = imports.decl_refs.get(&id) {
        for id in set {
          ref_counts.discount(id, |id| queue.push_back(id.clone()));
        }
      }
    }

    Self {
      names: removes.into_iter().collect(),
      ids: ref_counts.done,
    }
  }
}
