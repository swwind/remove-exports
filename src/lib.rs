use std::rc::Rc;

mod visitor;

use swc_common::{input::StringInput, BytePos, SourceMap};
use swc_ecmascript::{
  ast::EsVersion,
  codegen::{text_writer::JsWriter, Emitter},
  parser::{lexer::Lexer, EsConfig, Parser, Syntax},
  visit::{VisitMutWith, VisitWith},
};
use visitor::{CountVisitor, ImportVisitor, RemoveVisitor};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn remove_exports(source: &str, exports: Vec<String>) -> String {
  let lexer = Lexer::new(
    Syntax::Es(EsConfig::default()),
    EsVersion::EsNext,
    StringInput::new(source, BytePos(0), BytePos(source.as_bytes().len() as u32)),
    None,
  );

  let mut parser = Parser::new_from(lexer);
  let mut module = parser.parse_module().unwrap();
  // println!("{:?}", module);

  let mut first = ImportVisitor::default();
  module.visit_with(&mut first);
  // println!("{:?}", first);

  let mut counter = CountVisitor::default();
  module.visit_with(&mut counter);

  module.visit_mut_with(&mut RemoveVisitor {
    removes: exports.into_iter().collect(),
  });

  // println!("{:?}", module);

  let mut buf = vec![];
  {
    let cm = Rc::new(SourceMap::default());
    let mut emitter = Emitter {
      cfg: Default::default(),
      cm: cm.clone(),
      comments: None,
      wr: JsWriter::new(cm, "\n", &mut buf, None),
    };
    emitter.emit_module(&module).unwrap()
  }
  String::from_utf8_lossy(&buf).to_string()
}

#[cfg(test)]
mod tests;
