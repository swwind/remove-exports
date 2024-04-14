import { removeExports } from "../refs/example.ts";

console.log(
  removeExports(
    `
    export function foo() { bar() }
    export function bar() { foo() }
    // export { foo, bar };
    `,
    ["bar"]
  )
);
