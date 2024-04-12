import { removeExports } from "./example.ts";

console.log(
  removeExports(
    `
    const foo = 114;
    const bar = 514;
    const baz = [];
    const a = baz || foo;
    const b = baz || bar;
    export { a, b };
    `,
    ["a", "b"]
  )
);
