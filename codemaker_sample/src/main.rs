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

use codemaker::{CodeMaker, OutputFileSet};
use codemaker_sample::{PythonStatusModuleMaker, StatusCodes};

fn main() -> std::io::Result<()> {
    let codes = StatusCodes {
        codes: vec![
            (100, "Continue".into()),
            (200, "OK".into()),
            (404, "Not Found".into()),
        ],
    };
    let maker = PythonStatusModuleMaker {
        module_name: "status_codes".into(),
    };
    maker.make(&codes).write_into_dir("./")?;
    Ok(())
}
