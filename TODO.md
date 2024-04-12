# Issues

- [ ] Remove `foo` in `export const { foo, ...rest }` may cause rest value to change.

```js
var [a, b] = test();
var a = test();
var b = test();

export { a as foo };
// removes foo
var [, b] = test();
var b = test();
export { b as foo };
```
