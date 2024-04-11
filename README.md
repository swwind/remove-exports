# `@swwind/remove-exports`

Remove specific named exports in a JS file, also removes non-used imports and declarations caused by removal.

```tsx
// before

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

// after removing "findUser"

import { useState } from "frontend";
import "./style.css";

export function Component() {
  const [count, setCount] = useState(0);
  return <div>{count}</div>;
}

// "findUser" removes -> "USER_NAME" and "database" becomes unused, also removes
```

## API

```ts
import { remove_exports } from "@swwind/remove_exports";

const code = `export var foo, bar;`;
const result = remove_exports(code, ["foo"]);
// => `export var bar;`
```

## Docs

Among removal, those statements will not be removed:

```ts
// imports that has no references in removed exports
import "./style.css";
import { NeverUsed } from "./never-used.ts";

// expressions will never be removed
await database.initialize();

// declarations that has no references in removed exports
function frontend() {}
class Timer {}
```

Those will be removed.

```ts
// imports that only be referenced inside removed exports
import { database } from "./database.ts";

// declartions that only be referenced inside removed exports
function backend() {}

// matched exports
export function removedExports() {
  database;
  backend();
}
```
