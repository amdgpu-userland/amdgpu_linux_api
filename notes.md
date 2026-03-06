## Modeling ioctl structures
### Beware of enums
When you see a known set of values you might be tempted to use an
enum with repr and manual discriminants.

Assuming the api you build against is commited to backwards compatibility:
Your code could break, when it doesn't have to.

If in the future the thing you are writing bindings to
decided to add another variant you are not currently handling.

Rust's
```rust
#[non_exhaustive]
```
attribute doesn't help as it's more of a hint to users of your library.
