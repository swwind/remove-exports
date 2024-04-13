use std::collections::HashSet;

use swc_ecmascript::{
  ast::{Id, Ident},
  visit::{Visit, VisitWith},
};

/// Analyze Every Id
#[derive(Default, Debug)]
pub struct CountVisitor {
  pub set: HashSet<Id>,
}

impl CountVisitor {
  pub fn count<T>(expr: &T) -> HashSet<Id>
  where
    T: VisitWith<Self>,
  {
    let mut counter = Self::default();
    expr.visit_with(&mut counter);
    counter.set
  }
}

impl Visit for CountVisitor {
  fn visit_ident(&mut self, n: &Ident) {
    self.set.insert(n.to_id());
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn find_count() {
    assert_eq!(1 + 1, 2);
  }
}
