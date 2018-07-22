# is-match

A mini crate to check whether something matches something else:

```rust
let value = some_function_call();

if is_match!(value, Ok(Some(EnumFoo::ComplexThing { Some(_), .. })) | Ok(Some(EnumFoo::Bar(_)))) {
  /* do things */
}
```

# License

This project was extracted from [imag](https://imag-pim.org) at commit
[c74c26ccd143d905c94ecf84ac423293b7170623](http://git.imag-pim.org/imag/commit/?id=c74c26ccd143d905c94ecf84ac423293b7170623)
where only I was the author of this file.
imag itself is licensed as LGPL2.1, but I'm relicensing this piece of code as
MPL 2.0 here.

See [LICENSE](./LICENSE) - MPL 2.0

