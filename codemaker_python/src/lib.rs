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

//! # Generate Python code using the `codemaker` crate.
//!
//! This is an initial experiment in structural generation of Python
//! code from Rust. We'll see how it works out...

use codemaker::traits::*;

const INDENT: &'static str = "    ";

macro_rules! indented_writeln {
    ($writer:expr, $indent:expr, $fmt:literal $($tail:tt)*) => {
        writeln!($writer, concat!("{}", $fmt), INDENT.repeat($indent) $($tail)*)
    };
}

macro_rules! indented_write {
    ($writer:expr, $indent:expr, $fmt:literal $($tail:tt)*) => {
        write!($writer, concat!("{}", $fmt), INDENT.repeat($indent) $($tail)*)
    };
}

/// A Python package, the highest-level output format for Python code.

pub struct Package {
    dirpath: std::path::PathBuf,
    root_module: Module,
    submodules: Vec<Module>,
    subpackages: Vec<Package>,
}

impl Package {
    pub fn new<T: Into<String>>(name: T) -> Self {
        let name = name.into();
        Package {
            dirpath: std::path::PathBuf::from(name.clone()),
            root_module: Module::new("__init__").edit(|m| {
                m.filepath = std::path::PathBuf::from(name).join(m.filepath.as_path());
            }),
            submodules: vec![],
            subpackages: vec![],
        }
    }
}

impl codemaker::OutputFileSet for Package {
    type OutputFile = Module;
    fn files(&self) -> Vec<&Self::OutputFile> {
        vec![&self.root_module]
            .into_iter()
            .chain(self.submodules.iter())
            .chain(self.subpackages.iter().map(|p| p.files()).flatten())
            .collect()
    }
}

impl FluentAPI for Package {}

/// Adding Statements to a Package, puts them in its root module.
impl std::iter::Extend<Statement> for Package {
    fn extend<T: IntoIterator<Item = Statement>>(&mut self, iter: T) {
        for stmt in iter {
            self.root_module.statements.push(stmt);
        }
    }
}

/// Adding Modules to a Package, creates sub-modules.
impl std::iter::Extend<Module> for Package {
    fn extend<T: IntoIterator<Item = Module>>(&mut self, iter: T) {
        for mut m in iter {
            m.filepath = self.dirpath.join(m.filepath.as_path());
            self.submodules.push(m);
        }
    }
}

/// Adding Packages to a Package, creates sub-packages.
impl std::iter::Extend<Package> for Package {
    fn extend<T: IntoIterator<Item = Package>>(&mut self, iter: T) {
        for mut p in iter {
            // TODO: need to adjust nested filepaths, that's kinda gross...
            p.dirpath = self.dirpath.join(p.dirpath.as_path());
            self.subpackages.push(p);
        }
    }
}

/// A Python module, a single file containing Python source code.

pub struct Module {
    filepath: std::path::PathBuf,
    statements: Vec<Statement>,
}

impl Module {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        let name = name.as_ref();
        Module {
            filepath: std::path::PathBuf::from(format!("{}.py", name)),
            statements: vec![],
        }
    }
}

impl codemaker::OutputFile for Module {
    fn path(&self) -> &std::path::Path {
        self.filepath.as_path()
    }
    fn write_into<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for stmt in &self.statements {
            stmt.write_into(writer, 0)?
        }
        Ok(())
    }
}

impl FluentAPI for Module {}

impl std::iter::Extend<Statement> for Module {
    fn extend<I: IntoIterator<Item = Statement>>(&mut self, iter: I) {
        for item in iter {
            self.statements.push(item);
        }
    }
}

/// A Statement, any of a several kinds of executable chunk of Python source code.
///
/// This is an Enum to allow different kinds of Statement to be conveniently stored
/// in a single list. For actually builting the API, you almost certainly want to use
/// one of the contained types like [`Assignment`] or [`FunctionDefinition`].
pub enum Statement {
    Assign(Assignment),
    FuncDef(FunctionDefinition),
    Raw(String),
}

impl Statement {
    pub fn new_raw<T: Into<String>>(stmt: T) -> Statement {
        Statement::Raw(stmt.into())
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        match self {
            Self::Assign(a) => a.write_into(writer, indent)?,
            Self::FuncDef(f) => f.write_into(writer, indent)?,
            Self::Raw(ln) => indented_writeln!(writer, indent, "{}", ln)?,
        }
        Ok(())
    }
}

pub struct Assignment {
    target: String, // TODO, could also be item assigment etc
    value: String,  // TODO: should be generic "Expression" type.
}

impl Assignment {
    pub fn new<T1: Into<String>, T2: Into<String>>(target: T1, value: T2) -> Self {
        Assignment {
            target: target.into(),
            value: value.into(),
        }
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        indented_writeln!(writer, indent, "{} = {}", self.target, self.value)?;
        Ok(())
    }
}

impl Into<Statement> for Assignment {
    fn into(self) -> Statement {
        Statement::Assign(self)
    }
}

pub struct FunctionDefinition {
    name: String,
    args: Vec<String>, // TODO: a richer arg type, with defaults etc
    body: Vec<Statement>,
}

impl FunctionDefinition {
    pub fn new<T: Into<String>>(name: T) -> Self {
        FunctionDefinition {
            name: name.into(),
            args: vec![],
            body: vec![],
        }
    }

    pub fn edit<F: FnOnce(&mut Self)>(mut self, func: F) -> Self {
        func(&mut self);
        return self;
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        indented_write!(writer, indent, "def {}(", self.name)?;
        for arg in &self.args {
            write!(writer, "{},", arg)?;
        }
        writeln!(writer, "):")?;
        if self.body.is_empty() {
            indented_writeln!(writer, indent, "pass")?;
        } else {
            for stmt in &self.body {
                stmt.write_into(writer, indent + 1)?;
            }
        }
        Ok(())
    }

    pub fn add_arg<T: Into<String>>(mut self, name: T) -> Self {
        self.args.push(name.into());
        return self;
    }

    pub fn push<T: Into<Statement>>(mut self, stmt: T) -> Self {
        self.body.push(stmt.into());
        return self;
    }
}

impl Into<Statement> for FunctionDefinition {
    fn into(self) -> Statement {
        Statement::FuncDef(self)
    }
}

impl FluentAPI for FunctionDefinition {}

impl std::iter::Extend<Statement> for FunctionDefinition {
    fn extend<I: IntoIterator<Item = Statement>>(&mut self, iter: I) {
        for item in iter {
            self.body.push(item);
        }
    }
}
