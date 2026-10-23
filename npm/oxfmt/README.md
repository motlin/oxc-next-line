# oxfmt-next-line

A fork of [Oxfmt](https://oxc.rs/docs/guide/usage/formatter), the Oxc formatter, that puts opening braces on the next line.
It is the Oxfmt counterpart of [prettier-next-line](https://www.npmjs.com/package/prettier-next-line).

```js
function example(value)
{
  if (value)
  {
    return true;
  }
  else
  {
    return false;
  }
}

const settings =
{
  enabled: true,
};
import { alpha, beta } from "./letters";
items.forEach(() => {});
```

Every block's `{` starts its own line, and `else` / `catch` / `finally` start a new line.
Object literals and type literals always break, one member per line.
Imports, exports and destructuring stay inline while they fit and move `{` down once they wrap.
An empty `{}` stays on one line, except as the body of a function, class or statement.
`return {` stays together, since splitting it would change the program.
CSS, SCSS, Less, GraphQL and JSON follow the same rule.

## Usage

```sh
npx oxfmt-next-line
```

The package installs both `oxfmt-next-line` and `oxfmt` commands, and reads the usual `.oxfmtrc.json`.
Everything else works as in [Oxfmt's documentation](https://oxc.rs/docs/guide/usage/formatter).

## Options

`braceStyle` is the only option this fork adds:

- `"next-line"` (default): braces as above.
- `"same-line"`: Oxfmt's own, Prettier-compatible output.

```json
{
  "braceStyle": "same-line"
}
```

## Source

[motlin/oxc-next-line](https://github.com/motlin/oxc-next-line), whose `main` branch replays the brace changes onto [oxc-project/oxc](https://github.com/oxc-project/oxc).
MIT licensed, like Oxc.
