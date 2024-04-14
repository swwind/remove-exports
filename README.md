# `@swwind/remove-exports`

Remove specific named exports in a JS file, also removes non-used imports and declarations caused by removal.

Used for tree shaking server-specific codes in full-stack js frameworks.

```tsx
// === before ===

import { database } from "backend";
import { useState } from "frontend";
import "./style.css";

const USER_NAME = "admin";

export const findUser = async () => {
  return await database.findUser(USER_NAME);
};

export function Component() {
  const [count, setCount] = useState(0);
  return <div>{count}</div>;
}

// === after removing "findUser" ===

// import { database } from "backend";
import { useState } from "frontend";
import "./style.css";

// const USER_NAME = "admin";

// export const findUser = async () => {
//   return await database.findUser(USER_NAME);
// };

export function Component() {
  const [count, setCount] = useState(0);
  return <div>{count}</div>;
}
```

## API

```ts
import { remove_exports } from "@swwind/remove-exports";

const code = `export var foo, bar;`;
const result = remove_exports(code, ["foo"]);
// => `export var bar;`
```
