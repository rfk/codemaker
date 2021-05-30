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

    pub fn edit<F: FnOnce(&mut Self)>(mut self, func: F) -> Self {
        func(&mut self);
        return self
    }

    fn with_root_module<F: FnOnce(Module) -> Module>(mut self, func: F) -> Self {
        self.root_module = func(self.root_module);
        self
    }

    pub fn add_submodule(mut self, mut m: Module) -> Self {
        m.filepath = self.dirpath.join(m.filepath.as_path());
        self.submodules.push(m);
        return self;
    }

    pub fn add_subpackage(mut self, mut p: Package) -> Self {
        // TODO: need to adjust nested filepaths, that's kinda gross...
        p.dirpath = self.dirpath.join(p.dirpath.as_path());
        self.subpackages.push(p);
        return self;
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

    pub fn edit<F: FnOnce(&mut Self)>(mut self, func: F) -> Self {
        func(&mut self);
        return self
    }

    pub fn push(mut self, stmt: Statement) -> Self {
        self.statements.push(stmt);
        self
    }

    pub fn extend(mut self, stmts: impl Iterator<Item=Statement>) -> Self {
        self.statements.extend(stmts);
        self
    }
}

impl codemaker::OutputFile for Module {
    fn path(&self) -> &std::path::Path {
        self.filepath.as_path()
    }
    fn write_into<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for stmt in &self.statements {
            stmt.write_into(writer)?
        }
        Ok(())
    }
}

pub struct Statement {
    lines: Vec<String>,
}


impl Statement {
    pub fn new() -> Self {
        Statement { lines: vec![] }
    }

    pub fn push(mut self, line: String) -> Self {
        self.lines.push(line);
        self
    }

    fn write_into<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for ln in &self.lines {
            writeln!(writer, "{}", ln)?;
        }
        Ok(())
    }
}
