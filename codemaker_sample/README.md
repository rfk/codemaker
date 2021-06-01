# CodeMaker Sample App

This is a small sample program to try out the `codemaker` crates.
It reads a list of numeric codes and their corresponding status text
from the file `status_codes.yaml`, and uses that data to generate a
Python module named `status_codes.py` containing:

 - a named constant for each status message, mapping to the code.
 - a function `status_for_code` that will map a code to its status text.

That's not a very exciting piece of generated code, but it's a nice
little exercise in seeing whether this whole thing is a good idea.

To try it out, `cargo run` in this directory and them observe the
resulting `status_codes.py` file. Try editing `status_codes.yaml`
with your own entries and then regenerating the output! Wheeee!
