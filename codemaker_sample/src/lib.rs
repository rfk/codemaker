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

use codemaker::{define_codemaker_rules, CodeMaker, CodeMakerRule, Extend};
use codemaker_python as py;

pub struct StatusCodes {
    pub codes: Vec<(u16, String)>,
}

pub struct PythonStatusModuleMaker {
    pub module_name: String,
}

impl<'a> CodeMaker<'a> for PythonStatusModuleMaker
{
    type Input = &'a StatusCodes;
    type Output = py::Module;
}

define_codemaker_rules! {
    PythonStatusModuleMaker as self {

        /// The main top-level conversion.
        &StatusCodes as input => py::Module {
            py::Module::new(self.module_name.as_str())
                .extend(self.make_from_iter(input.codes.iter()))
                .push(MakeCodeLookupFunc::make_from(input))
        }

        // Each individiual code entry becomes a global variable assignment.
        &(u16, String) as (code, name) => py::Assignment {
            py::Assignment::new(name.clone(), format!("{}", code))
        }
    }
}

pub struct MakeCodeLookupFunc { }

impl MakeCodeLookupFunc {
    fn make_from<Input, Output>(input: Input) -> Output
    where
        Self: CodeMakerRule<Input, Output>
    {
        CodeMakerRule::make_from(&MakeCodeLookupFunc {}, input)
    }
}

define_codemaker_rules! {
    MakeCodeLookupFunc as self {

        &StatusCodes as input => py::FunctionDefinition {
            py::FunctionDefinition::new("status_for_code".into())
                .add_arg("code".into())
                .extend(self.make_from_iter(input.codes.iter()))
        }

        &(u16, String) as input => py::Statement {
            py::Statement::Raw(format!("if code == {}: return \"{}\"", input.0, input.1))
        }
    }
}