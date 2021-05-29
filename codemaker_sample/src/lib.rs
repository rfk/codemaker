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

use codemaker::{define_codemaker_rules, CodeMaker};
use codemaker_python as py;

pub struct StatusCodes {
    pub codes: Vec<(u16, String)>,
}
pub struct PythonStatusModuleMaker {
    pub module_name: String,
}

impl<'a> CodeMaker<'a> for PythonStatusModuleMaker {
    type Input = &'a StatusCodes;
    type Output = py::Module;
}

define_codemaker_rules! {
    PythonStatusModuleMaker as self {

        // TODO: ideally we'd accept doc-comments here.

        // The main top-level conversion.
        &'a StatusCodes as input => py::Module {
            py::Module::new(self.module_name.as_str())
                .extend(self.make_from(input.codes.iter()))
        }

        // Each individiual code entry becomes a global variable assignment.
        &'a (u16, String) as input => py::Block {
            py::Block::new().push(format!("{} = {}", input.1, input.0))
        }
    }
}
