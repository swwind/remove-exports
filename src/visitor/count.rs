use std::collections::HashMap;

use swc_ecmascript::{
  ast::{FnDecl, Id, Ident},
  visit::Visit,
};

#[derive(Default, Debug)]
struct Counter<T> {
  counts: HashMap<T, u32>,
}

impl<T> Counter<T>
where
  T: std::hash::Hash + Eq,
{
  fn count(&mut self, key: T) {
    self.counts.entry(key).and_modify(|x| *x += 1).or_insert(1);
  }

  fn get(&self, key: &T) -> u32 {
    *self.counts.get(key).unwrap_or(&0)
  }
}

#[derive(Default, Debug)]
pub struct CountVisitor {
  counter: Counter<Id>,
}

impl Visit for CountVisitor {
  fn visit_ident(&mut self, n: &Ident) {
    self.counter.count(n.to_id());
  }

  fn visit_fn_decl(&mut self, n: &FnDecl) {}
}
