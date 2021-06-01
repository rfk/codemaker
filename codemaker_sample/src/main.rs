/* Copyright 2021 Ryan F Kelly
 *
 * Licensed under the Apache License (Version 2.0), or the MIT license,
 * (the "Licenses") at your option. You may not use this file except in
 * compliance with one of the Licenses. You may obtain copies of the
 * Licenses at:
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *    http://opensource.org/licenses/MIT
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the Licenses is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the Licenses for the specific language governing permissions and
 * limitations under the Licenses. */

//! A small sample program to try out `codemaker` crates.
//!
//! This is a small program that reads a list of numeric codes and their
//! corresponding status text from a file `status_codes.yaml`, and uses
//! that data to generate a Python module containing:
//!
//!  - a named constant for each status message, mapping to the code.
//!  - a function `status_for_code` that will map a code to its status text.
//!
//! That's not a very exciting piece of generated code, but it's a nice
//! little exercise in seeing whether this whole thing is a good idea.

use codemaker::{CodeMaker, OutputFileSet};
use codemaker_sample::{StatusModuleMaker, StatusCodes};

fn main() -> std::io::Result<()> {
    // Read the input data into our source data structure.
    let codes: StatusCodes = serde_yaml::from_str(
        std::fs::read_to_string("status_codes.yaml")
            .unwrap()
            .as_str(),
    )
    .unwrap();

    // Configure the Maker with the name of the output module.
    let maker = StatusModuleMaker {
        module_name: "status_codes".into(),
    };

    // Convert the input data into a Python module.
    let output = maker.make(&codes);

    // Wwrite it out to disk.
    output.write_into_dir("./")?;
    Ok(())
}
