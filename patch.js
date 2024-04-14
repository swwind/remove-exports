import { readFile, unlink, writeFile } from "node:fs/promises";

const code = `
const isNode = typeof process !==  "undefined";
if (isNode) {
  const fs = await import("node:fs/promises");
  input = await fs.readFile(input);
} else {
  input = fetch(input);
}
`;

const filepath = "./pkg/remove_exports.js";
const source = await readFile(filepath, "utf8");
await writeFile(filepath, source.replace("input = fetch(input);", code));

await unlink("./pkg/package.json");
await unlink("./pkg/README.md");
await unlink("./pkg/.gitignore");
