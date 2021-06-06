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

pub use codemaker_python_macros::quoted_rule;

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
    IfElse(IfElse),
    Return(Return),
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
            Self::IfElse(ie) => ie.write_into(writer, indent)?,
            Self::Return(r) => r.write_into(writer, indent)?,
            Self::Raw(ln) => indented_writeln!(writer, indent, "{}", ln)?,
        }
        Ok(())
    }
}

pub struct Assignment {
    target: String, // TODO: could also be item assigment etc
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

pub struct Return {
    value: Expression,
}

impl Return {
    pub fn new<T: Into<Expression>>(value: T) -> Self {
        Return {
            value: value.into(),
        }
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        indented_write!(writer, indent, "return ")?;
        self.value.write_into(writer)?;
        writeln!(writer, "")?;
        Ok(())
    }
}

impl Into<Statement> for Return {
    fn into(self) -> Statement {
        Statement::Return(self)
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

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        indented_write!(writer, indent, "def {}(", self.name)?;
        let mut args = self.args.iter();
        // Writing args, with nice commas.
        match args.next() {
            None => (),
            Some(arg) => {
                write!(writer, "{}", arg)?;
                for arg in args {
                    write!(writer, ", {}", arg)?;
                }
            }
        }
        writeln!(writer, "):")?;
        if self.body.is_empty() {
            indented_writeln!(writer, indent + 1, "pass")?;
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

    pub fn add_args<T: Into<String>, I: IntoIterator<Item=T>>(mut self, name: I) -> Self {
        self.args.extend(name.into_iter().map(Into::into));
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



#[derive(Default)]
pub struct Block {
    body: Vec<Statement>,
}

impl Block {
    pub fn new() -> Self {
        Block { body: vec![] }
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        if self.body.is_empty() {
            indented_writeln!(writer, indent, "pass")?;
        } else {
            for stmt in &self.body {
                stmt.write_into(writer, indent)?;
            }
        }
        Ok(())
    }
}

impl Block {
    fn is_empty(&self) -> bool {
        self.body.is_empty()
    }
}

impl FluentAPI for Block {}

impl std::iter::Extend<Statement> for Block {
    fn extend<I: IntoIterator<Item = Statement>>(&mut self, iter: I) {
        for item in iter {
            self.body.push(item);
        }
    }
}

pub struct IfElse {
    condition: Expression,
    body_if: Block,
    body_else: Block,
}

impl IfElse {
    pub fn new<T: Into<Expression>>(condition: T) -> Self {
        IfElse {
            condition: condition.into(),
            body_if: Block::new(),
            body_else: Block::new(),
        }
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W, indent: usize) -> std::io::Result<()> {
        indented_write!(writer, indent, "if ")?;
        self.condition.write_into(writer)?;
        writeln!(writer, ":")?;
        self.body_if.write_into(writer, indent + 1)?;
        if ! self.body_else.is_empty() {
            indented_writeln!(writer, indent, "else:")?;
            self.body_else.write_into(writer, indent + 1)?;
        }
        Ok(())
    }

    pub fn with_body_if<F>(mut self, func: F) -> Self
    where
        F: FnOnce(Block) -> Block
    {
        self.body_if = func(std::mem::take(&mut self.body_if));
        self
    }

    pub fn with_body_else<F>(mut self, func: F) -> Self
    where
        F: FnOnce(Block) -> Block
    {
        self.body_else = func(std::mem::take(&mut self.body_else));
        self
    }
}

impl Into<Statement> for IfElse {
    fn into(self) -> Statement {
        Statement::IfElse(self)
    }
}

impl FluentAPI for IfElse {}


pub enum Expression {
    Equals(Box<Expression>, Box<Expression>),
    Literal(String),
    Variable(String),
}

impl Expression {
    pub fn new_equals<T1: Into<Expression>, T2: Into<Expression>>(lhs: T1, rhs: T2) -> Self {
        Expression::Equals(Box::new(lhs.into()), Box::new(rhs.into()))
    }
    pub fn new_variable<T: Into<String>>(name: T) -> Self {
        Expression::Variable(name.into())
    }
    fn write_into<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Self::Equals(lhs, rhs) => {
                lhs.write_into(writer)?;
                write!(writer, " == ")?;
                rhs.write_into(writer)?;
            },
            Self::Literal(lit) => write!(writer, "{}", lit)?,
            Self::Variable(name) => write!(writer, "{}", name)?,
        }
        Ok(())
    }
}

impl From<&str> for Expression {
    fn from(value: &str) -> Expression {
        // TODO: escaping etc
        Expression::Literal(format!("\"{}\"", value))
    }
}

impl From<&String> for Expression {
    fn from(value: &String) -> Expression {
        // TODO: escaping etc
        Expression::Literal(format!("\"{}\"", value))
    }
}

impl From<&u16> for Expression {
    fn from(value: &u16) -> Expression {
        Expression::Literal(format!("{}", value))
    }
}