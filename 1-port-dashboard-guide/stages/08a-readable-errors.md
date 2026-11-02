# Stage 8a: Readable errors (special stage)

**Goal:** no development `{:?}` output is left, and every error your tool can print reads like a sentence.

This stage is an addition to the original guide. It sits here because Stages 3 to 8 are where your error types pile up, and Stage 9 is the first version other people will use.

## Background

### `Debug` and `Display`

| Placeholder | Trait | Meant for | Who writes it |
|---|---|---|---|
| `{:?}` | `Debug` | Programmers | The compiler, through `#[derive(Debug)]` |
| `{}` | `Display` | People using the tool | You, by hand |

`Debug` output exposes internal names, and its format is not a promise. `Display` is the wording you choose.

### What stays and what goes

`#[derive(Debug)]` stays on your types. `assert_eq!` and `.unwrap()` need it, so tests depend on it. What goes is `{:?}` used as program output.

One exception: `{:?}` is fine on a single value inside a `Display` message. It puts quotes around a string, which makes empty or odd text visible.

### Implementing `Display`

`Display` has one method, `fmt`. It usually holds a `match` with one arm per variant, each calling `write!`. Once you have it, `{}` works on your type, and so does `.to_string()`, which makes the wording easy to test.

`std::error::Error` marks a type as a real error, so it works with `?` and `Box<dyn Error>`. It needs `Debug` and `Display` to exist first, and then the implementation can be empty.

## Your task

1. Find every use: `grep -n '{:?}' src/*.rs`.
2. Implement `Display` and `std::error::Error` for each of your error enums.
3. Add a test per variant that checks the wording with `.to_string()`.
4. Change error output to use `{}`. Include the input that failed, so a message can be acted on.
5. Remove the remaining `{:?}` output. Sockets still printed this way can stay until Stage 9 replaces them with a table, but keep their `TODO` until then.
6. Remove the `TODO` comments this stage resolves, and the Debug note in your project's README once nothing temporary is left.

## Generic example

Errors from parsing a weather reading:

```rust
use std::fmt;

#[derive(Debug, PartialEq)]
enum ReadingError {
    Missing(&'static str),
    BadNumber { field: &'static str, text: String },
}

impl fmt::Display for ReadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadingError::Missing(field) => write!(f, "missing field: {field}"),
            ReadingError::BadNumber { field, text } => {
                write!(f, "{field} is not a valid number: {text:?}")
            }
        }
    }
}

impl std::error::Error for ReadingError {}

fn main() {
    let missing = ReadingError::Missing("humidity");
    let bad = ReadingError::BadNumber { field: "pressure", text: "abc".to_string() };

    assert_eq!(missing.to_string(), "missing field: humidity");
    assert_eq!(bad.to_string(), "pressure is not a valid number: \"abc\"");

    println!("{missing:?}"); // Missing("humidity")
    println!("{bad}");       // pressure is not a valid number: "abc"
}
```

## Verify

```sh
cargo test
cargo clippy --all-targets
grep -n '{:?}' src/*.rs   # only intentional uses remain
```

Then trigger a real failure, for example by temporarily changing the path in your reader. Read the message as if you had never seen the code. Does it say what went wrong?

## Pitfalls

- **Removing `#[derive(Debug)]`.** Your tests stop compiling. Keep the derive and change only the output.
- **Calling `{}` on the value inside its own `Display`.** Writing `self` with `{}` inside `fmt` calls `fmt` again, forever. Write the fields instead.
- **Messages with no context.** `invalid number` tells the reader nothing. Name the field and include the text.

## Done when

- [ ] Every error enum implements `Display` and `std::error::Error`
- [ ] A test checks the wording of each variant
- [ ] `grep -n '{:?}' src/*.rs` shows only intentional uses
- [ ] `cargo clippy --all-targets` is clean
- [ ] Committed
