# CodeMaker

This is a very early-days experiment in doing structured data-based
generation of source code with Rust. It currently has a sketch of
the basic crates, and a small sample app in `./codemaker_sample` that
might give a bit of an idea of it fits together. If you're interested
in taking an early look, please start from the sample app and work
your way down into the details:

```
cd ./codemaker_sample
cat status_codes.yaml
cargo run
cat status_codes.py
```

There's not much about the framework here that's specific to generating
source code as distinct from other output formats, it's just that the
ideas are inspired by the sorts of code-generation that we've been
doing at Mozilla for projects like [UniFFI](https://github.com/mozilla/uniffi-rs/)
and [Glean](https://github.com/mozilla/glean/). There are no plans to
replace the code-generation backends of those projects with this work,
it's just me toying around with some ideas. (But hey, if they turn out
to be *good* ideas, then who knows :grin:).