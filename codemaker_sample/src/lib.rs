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

//! # A sample crate that uses `codemaker`.
//#![feature(trace_macros)]

use heck::ShoutySnakeCase;
use serde::Deserialize;

use codemaker::{define_codemaker_rules, define_stateless_codemaker, CodeMaker};
use codemaker_python as py;

/// This is our input data, a list of numeric codes and their
/// corresponding status message. A real app would work with something
/// a good deal more elaborate than this of course.
#[derive(Debug, Deserialize)]
pub struct StatusCodes {
    pub codes: Vec<(u16, String)>,
}

/// This is our top-level "Maker", which knows how to convert
/// the input data into a Python module.
pub struct StatusModuleMaker {
    pub module_name: String,
}

impl<'a> CodeMaker<'a> for StatusModuleMaker {
    type Input = &'a StatusCodes;
    type Output = py::Module;
}

// These are the rules by which to convert input to output.
define_codemaker_rules! {
    StatusModuleMaker as self {

        /// The main top-level conversion.
        /// This makes the python module, pushing a definition for each
        /// individual code plus
        &StatusCodes as input => py::Module {
            py::Module::new(self.module_name.as_str())
                .extend(self.make_from_iter(input.codes.iter()))
                .push(CodeLookupFunc::make_from(input))
        }

        /// Each individiual code entry becomes a global variable assignment,
        /// with its name converted to a proper python variable name. A future
        /// enhancement might has a helper like `py::Constant` or similar that
        /// can do the idiomatic casing for you transparently.
        &(u16, String) as (code, name) => py::Assignment {
            py::Assignment::new(name.to_shouty_snake_case(), format!("{}", code))
        }
    }
}

//trace_macros!(true);

define_stateless_codemaker! {
    /// Make a function for looking up the string for an integer status code.
    ///
    /// This generates a python function named `status_for_code` which will accept
    /// an integer status code and return the corresponding status string, or None
    /// if the code is unknown.
    CodeLookupFunc {
        &StatusCodes as input => @py::quoted_rule: FunctionDefinition! {
            def status_for_code(code):{
                $(Self::make_from_iter(&input.codes))*
                return "" // TODO: macro-based support for variables, to return `None`
            }
        }
        &(u16, String) as (code, status) => @py::quoted_rule: Statement! {
            if code == $(code):{
                return $(status)
            }
        }
    }
}
