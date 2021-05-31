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

use codemaker::FluentAPI;

const INDENT: &'static str = "    ";

pub struct Package {
    dirpath: std::path::PathBuf,
    root_module: Module,
    submodules: Vec<Module>,
    subpackages: Vec<Package>,
}

impl Package {
    pub fn new(name: &str) -> Self {
        Package {
            dirpath: std::path::PathBuf::from(name),
            root_module: Module::new("__init__").edit(|m| {
                m.filepath =  std::path::PathBuf::from(name).join(m.filepath.as_path());
            }),
            submodules: vec![],
            subpackages: vec![],
        }
    }

    fn with_root_module<F: FnOnce(Module) -> Module>(mut self, func: F) -> Self {
        self.root_module = func(self.root_module);
        self
    }
}

impl FluentAPI for Package {}

impl Extend<Module> for Package {
    fn extend<T: IntoIterator<Item=Module>>(&mut self, iter: T) {
        for mut m in iter {
            m.filepath = self.dirpath.join(m.filepath.as_path());
            self.submodules.push(m);
        }
    }
}

impl Extend<Package> for Package {
    fn extend<T: IntoIterator<Item=Package>>(&mut self, iter: T) {
        for mut p in iter {
            // TODO: need to adjust nested filepaths, that's kinda gross...
            p.dirpath = self.dirpath.join(p.dirpath.as_path());
            self.subpackages.push(p);
        }
    }
}

impl codemaker::OutputFileSet for Package {
    type OutputFile = Module;
    fn files(&self) -> Vec<&Self::OutputFile> {
        vec![&self.root_module].into_iter().chain(
            self.submodules.iter()
        ).chain(
            self.subpackages.iter().map(|p| p.files()).flatten()
        ).collect()
    }
}

pub struct Module {
    filepath: std::path::PathBuf,
    statements: Vec<Statement>,
}

impl Module {
    pub fn new(name: &str) -> Self {
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

impl Extend<Statement> for Module {
    fn extend<I: IntoIterator<Item=Statement>>(&mut self, iter: I) {
        for item in iter {
            self.statements.push(item);
        }
    }
}


pub enum Statement {
    AssignStmt(Assignment),
    FuncDefStmt(FunctionDefinition),
    Raw(String),
}


impl Statement {
    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        match self {
            Self::AssignStmt(a) => a.write_into(writer, indent)?,
            Self::FuncDefStmt(f) => f.write_into(writer, indent)?,
            Self::Raw(ln) => writeln!(writer, "{}{}", INDENT.repeat(indent), ln)?,
        }
        Ok(())
    }
}

pub struct Assignment {
    target: String,  // TODO, could also be item assigment etc
    value: String,   // TODO: should be generic "Expression" type.
}

impl Assignment {
    pub fn new(target: String, value: String) -> Self {
        Assignment { target, value }
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        writeln!(writer, "{}{} = {}", INDENT.repeat(indent), self.target, self.value)?;
        Ok(())
    }
}

impl Into<Statement> for Assignment {
    fn into(self) -> Statement {
        Statement::AssignStmt(self)
    }
}


pub struct FunctionDefinition {
    name: String,
    args: Vec<String>,      // TODO: a richer arg type, with defaults etc
    body: Vec<Statement>,
}

impl FunctionDefinition {
    pub fn new(name: String) -> Self {
        FunctionDefinition { name, args: vec![], body: vec![] }
    }

    pub fn edit<F: FnOnce(&mut Self)>(mut self, func: F) -> Self {
        func(&mut self);
        return self
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        write!(writer, "{}def {}(", INDENT.repeat(indent), self.name)?;
        for arg in &self.args {
            write!(writer, "{},", arg)?;
        }
        writeln!(writer, "):")?;
        if self.body.is_empty() {
            writeln!(writer, "{}pass", INDENT.repeat(indent+1))?;
        } else {
            for stmt in &self.body {
                stmt.write_into(writer, indent + 1)?;
            }
        }
        Ok(())
    }

    pub fn add_arg(mut self, name: String) -> Self {
        self.args.push(name);
        return self;
    }

    pub fn push<T: Into<Statement>>(mut self, stmt: T) -> Self {
        self.body.push(stmt.into());
        return self;
    }
}

impl Into<Statement> for FunctionDefinition {
    fn into(self) -> Statement {
        Statement::FuncDefStmt(self)
    }
}

impl FluentAPI for FunctionDefinition {}

impl Extend<Statement> for FunctionDefinition {
    fn extend<I: IntoIterator<Item=Statement>>(&mut self, iter: I) {
        for item in iter {
            self.body.push(item);
        }
    }
}