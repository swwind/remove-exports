use std::rc::Rc;

#[cfg(test)]
mod test;
mod visitor;

use swc_common::{
  comments::SingleThreadedComments, input::SourceFileInput, FileName, Mark, SourceMap,
};
use swc_common::{Globals, GLOBALS};
use swc_ecmascript::transforms::resolver;
use swc_ecmascript::{
  ast::EsVersion,
  codegen::{text_writer::JsWriter, Emitter},
  parser::{lexer::Lexer, EsConfig, Parser, Syntax},
  visit::{VisitMutWith, VisitWith},
};
use visitor::{ImportVisitor, RemoveVisitor};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn remove_exports(source: &str, exports: Vec<String>) -> String {
  let cm = Rc::new(SourceMap::default());
  let fm = cm.new_source_file(FileName::Custom("input.js".to_string()), source.to_string());

  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(
    Syntax::Es(EsConfig::default()),
    EsVersion::Es2022,
    SourceFileInput::from(&*fm),
    Some(&comments),
  );

  let mut parser = Parser::new_from(lexer);
  let mut module = parser.parse_module().unwrap();

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let mut resolver = resolver(Mark::new(), Mark::new(), false);
    module.visit_mut_with(&mut resolver);
  });

  // println!("=============");
  // println!("before = {:?}", module);

  let mut import = ImportVisitor::default();
  module.visit_with(&mut import);
  println!("import = {:?}", import);

  let mut remove = RemoveVisitor::new(import, exports);
  // println!("remove = {:?}", remove);
  module.visit_mut_with(&mut remove);

  // println!("module = {:?}", module);

  // println!("{:?}", module);

  let mut buf = vec![];
  {
    let mut emitter = Emitter {
      cfg: Default::default(),
      cm: cm.clone(),
      comments: Some(&comments),
      wr: JsWriter::new(cm, "\n", &mut buf, None),
    };
    emitter.emit_module(&module).unwrap()
  }
  String::from_utf8_lossy(&buf).to_string()
}
