import { remove_exports } from "./index.js";
import test from "node:test";
import { equal as assertEquals, throws as assertThrows } from "node:assert";

test("should work", () => {
  const code = `
  import { database } from "sqlite";
  const USER = 114514;
  export const foo = () => database(USER);
  export default USER;
  `;
  const expected = `const USER = 114514;
export default USER;
`;

  const result = remove_exports(code, ["foo"]);

  assertEquals(result, expected);
});

test("should throw on invalid code", () => {
  const code = `
  a7268t612t*^T@&^(!%&^T@^!!R&@^(TR^!@T(*R!TR%)!@&%^(*@&!%(@!)()()())
  `;

  assertThrows(() => remove_exports(code, []));
  assertThrows(() => remove_exports(code, []));
  assertThrows(() => remove_exports(code, []));
  assertEquals(remove_exports(`export const foo = bar();`, ["foo"]), "");
});
