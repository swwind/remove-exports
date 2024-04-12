use crate::remove_exports;

macro_rules! run {
  ($src:expr, $rms:expr, $ept:expr) => {{
    let removes = $rms.into_iter().map(|x| x.to_string()).collect();
    assert_eq!(remove_exports($src, removes).trim(), $ept)
  }};
}

macro_rules! run_empty {
  ($src:expr,  $rms:expr) => {
    run!($src, $rms, "")
  };
}

macro_rules! run_test {
  ($name:expr) => {{
    let source = include_str!(concat!("../tests/", $name, "/input.js"));
    let expected = include_str!(concat!("../tests/", $name, "/output.js"));
    let removes = include_str!(concat!("../tests/", $name, "/removes.txt"));
    let removes = removes
      .split_terminator('\n')
      .map(|x| x.to_string())
      .collect();

    assert_eq!(remove_exports(source, removes), expected,);
  }};
}

#[test]
fn remove_export_default_expr() {
  run_empty!("export default 2333;", ["default"]);
  run_empty!("export default foobar;", ["default"]);
  run_empty!("export default (class {});", ["default"]);
  run_empty!("export default (class foo {});", ["default"]);
  run_empty!("export default (function () {});", ["default"]);
  run_empty!("export default (function foo() {});", ["default"]);
}

#[test]
fn remove_export_default_decl() {
  run_empty!("export default class {}", ["default"]);
  run_empty!("export default class foo {}", ["default"]);
  run_empty!("export default function () {}", ["default"]);
  run_empty!("export default function* () {}", ["default"]);
  run_empty!("export default function foo() {}", ["default"]);
  run_empty!("export default function* foo() {}", ["default"]);
  run_empty!("export default async function () {}", ["default"]);
  run_empty!("export default async function* () {}", ["default"]);
  run_empty!("export default async function foo() {}", ["default"]);
  run_empty!("export default async function* foo() {}", ["default"]);
}

#[test]
fn remove_export_decl() {
  run_empty!("export var foo = null", ["foo"]);
  run_empty!("export let foo = null", ["foo"]);
  run_empty!("export const foo = null", ["foo"]);

  run!(
    "export const foo = 1, bar = 2",
    ["foo"],
    "export const bar = 2;"
  );

  // array pat
  run!(
    "export var [foo, , bar = 233, ...baz] = []",
    ["foo"],
    "export var [, , bar = 233, ...baz] = [];"
  );
  run!(
    "export var [foo, , bar = 233, ...baz] = []",
    ["bar"],
    "export var [foo, , , ...baz] = [];"
  );
  run!(
    "export var [foo, , bar = 233, ...baz] = []",
    ["baz"],
    "export var [foo, , bar = 233, ] = [];"
  );
  run!(
    "export var [foo, , bar = 233, ...baz] = []",
    ["foo", "bar", "baz"],
    ""
  );

  // object pat
  run!(
    "export const { foo, bar: baz, bai = 2333, ...rest } = {}",
    ["foo"],
    "export const { bar: baz, bai = 2333, ...rest } = {};"
  );
  run!(
    "export const { foo, bar: baz, bai = 2333, ...rest } = {}",
    ["bar"],
    "export const { foo, bar: baz, bai = 2333, ...rest } = {};"
  );
  run!(
    "export const { foo, bar: baz, bai = 2333, ...rest } = {}",
    ["baz"],
    "export const { foo, bai = 2333, ...rest } = {};"
  );
  run!(
    "export const { foo, bar: baz, bai = 2333, ...rest } = {}",
    ["bai"],
    "export const { foo, bar: baz, ...rest } = {};"
  );
  run!(
    "export const { foo, bar: baz, bai = 2333, ...rest } = {}",
    ["rest"],
    "export const { foo, bar: baz, bai = 2333 } = {};"
  );
  run!(
    "export const { foo, bar: baz, bai = 2333, ...rest } = {}",
    ["foo", "baz", "bai", "rest"],
    ""
  );

  // functions
  run!("export function foo() {}", ["foo"], "");
  run!(
    "export function bar() {}",
    ["foo"],
    "export function bar() {}"
  );
  run!("export function* foo() {}", ["foo"], "");
  run!(
    "export function* bar() {}",
    ["foo"],
    "export function* bar() {}"
  );

  // class
  run!("export class foo {}", ["foo"], "");
  run!("export class bar {}", ["foo"], "export class bar {\n}");

  // export all
  run!("export * as foo from \"source\";", ["foo"], "");
}

#[test]
fn remove_export_names() {
  run!("export { foo }", ["foo"], "");
  run!("export { foo as bar }", ["foo"], "export { foo as bar };");
  run!("export { bar as foo }", ["foo"], "");

  run!("export { foo, bar }", ["foo"], "export { bar };");
  run!("export { foo, bar }", ["bar"], "export { foo };");
  run!("export { foo, bar }", ["baz"], "export { foo, bar };");
  run!("export { foo, bar }", ["foo", "bar"], "");

  run!("export { foo as \"😀\" } ", ["😀"], "");
}

#[test]
fn remove_infected_imports() {
  run!(
    r#"
    import { bar } from "source";
    export function foo() { bar; }
    "#,
    ["foo"],
    ""
  );

  run!(
    r#"
    import { bar } from "source";
    bar;
    export function foo() { bar; }
    "#,
    ["foo"],
    "import { bar } from \"source\";\nbar;"
  );

  run!(
    r#"
    import { bar } from "source";
    export function foo(bar) { bar; }
    "#,
    ["foo"],
    "import { bar } from \"source\";"
  );

  run!(
    r#"
    import { bar } from "source";
    export function foo() { var bar; bar = 2; }
    "#,
    ["foo"],
    "import { bar } from \"source\";"
  );

  run!(
    r#"
    import {} from "source";
    export function foo() {}
    "#,
    ["foo"],
    "import \"source\";"
  );

  run!(
    r#"
    import "source";
    export function foo() {}
    "#,
    ["foo"],
    "import \"source\";"
  );
}

#[test]
fn remove_infected_decls() {
  run!(
    r#"
    const bar = 233;
    export function foo() { return bar; }
    "#,
    ["foo"],
    ""
  );

  run!(
    r#"
    function bar() { return foo(); }
    export function foo() { return bar(); }
    "#,
    ["foo"],
    ""
  );

  run!(
    r#"
    const bar = 233;
    export function foo(bar) {}
    "#,
    ["foo"],
    "const bar = 233;"
  );

  run!(
    r#"
    const bar = 233;
    export function foo(bar = bar) {}
    "#,
    ["foo"],
    "const bar = 233;"
  );

  run!(
    r#"
    import { baka } from "source";
    const baz = (foo, bar) => baka(foo, bar);
    const bar = (foo) => baz(bar, foo);
    export function foo() { bar(foo) }
    "#,
    ["foo"],
    ""
  );

  run!(
    r#"
    import { baka } from "source";
    const baz = (foo, bar) => baka(foo, bar);
    export const bar = (foo) => baz(bar, foo);
    export function foo() { bar(foo) }
    "#,
    ["foo"],
    "import { baka } from \"source\";\nconst baz = (foo, bar) => baka(foo, bar);\nexport const bar = (foo) => baz(bar, foo);"
  );
}
