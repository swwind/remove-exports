import { removeExports } from "./example.ts";

console.log(
  removeExports(
    `
    import { database } from "backend";
    import { useState } from "frontend";
    import "./style.css";

    const USER_NAME = "admin";

    export const findUser = async () => {
      return await database.findUser(USER_NAME);
    };

    export function Component() {
      const [count, setCount] = useState(0);
      return h("div", null, [ count ]);
    }
    `,
    ["findUser"]
  )
);
