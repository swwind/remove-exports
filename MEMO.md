## Implements

1.  Find all import idents and module-level decls.

    ```ts
    import foo from "source"; // create (foo)
    import { foo } from "source"; // create (foo)
    import { bar as foo } from "source"; // create (foo)
    import * as foo from "source"; // create (foo)

    function foo() {} // create (foo)
    function* foo() {} // create (foo)
    class foo {} // create (foo)

    export function foo() {} // create (foo) and mark as exported
    export function* foo() {} // create (foo) and mark as exported
    export class foo {} // create (foo) and mark as exported

    const foo; // create (foo)
    let foo; // create (foo)
    var foo; // create (foo)

    export { foo }; // mark (foo) as exported
    export { bar as foo }; // mark (bar) as exported
    ```

2.  Analyze dependency graph.

    ```ts
    const bar = 2333;
    const foo = bar + 2444; // link (foo) -> (bar)

    export function baz(a = baz) {}
    export function baz(a = bar) {} // link (baz) -> (bar)
    export function baz(a = foo) {} // link (baz) -> (foo)
    ```

3.  Recursively find decls should be removed.

    ```ts
    import { baz } from "baz"; // should remove
    const bar = baz + 1; // should remove
    export const foo = bar + 1; // should remove

    // == another file ==

    import { baz } from "baz"; // reserved
    export const bar = baz + 1; // reserved
    export const foo = bar + 1; // should remove
    ```

4.  Non-related statements / module decls

    ```ts
    // empty imports
    import {} from "dotenv";
    import "dotenv";

    // unknown exports
    export * from "source";

    // expressions
    console.log(114514);
    debugger;
    ```
