# Craban 🦀

Build a dependancy graph of a ES6 project

## Examples

<https://github.com/microsoft/TypeScript-Node-Starter/tree/master/src>
![Example](./img/out.png)

<https://github.com/nestjs/nest> (1K+ files)

```bash
# execution time
0.56s user 0.28s system 83% cpu 1.010 total
```

[SVG](https://www.figma.com/file/OAyJnZ5Pr0c6jnl5IxQmJd/Untitled?node-id=1%3A2&t=fBY2qmekHhwzRFq9-1)

## Usage

```bash
Usage: craban [-d <directory>]
       craban [--help] [-h]
```

```bash
craban -d assets/TypeScript-Node-Starter/src

# generate dot file
dot -Grankdir=LR -Tpng example1.dot -oout.png
```

## Limitations:

- Does not support absolute path imports

```typescript
import myFunc from "src/utils/file";
```

- Only supports ES6 module imports
