import { removeExports } from "./example.ts";

console.log(
  removeExports(
    `
    export default function foo() {}
    foo();
    `,
    ["default"]
  )
);
