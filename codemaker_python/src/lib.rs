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
    modules: Vec<Module>,
    // TODO: could also contain sub-packages
}

impl Package {
    pub fn new(name: &str) -> Self {
        Package {
            dirpath: std::path::PathBuf::from(name),
            modules: vec![],
        }
    }

    pub fn add_module(mut self, mut m: Module) -> Self {
        m.filepath = self.dirpath.join(m.filepath);
        self.modules.push(m);
        return self;
    }
}

impl codemaker::OutputFileSet for Package {
    type OutputFile = Module;
    fn files(&self) -> Vec<&Self::OutputFile> {
        self.modules.iter().collect()
    }
}

pub struct Module {
    filepath: std::path::PathBuf,
    blocks: Vec<Block>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Module {
            filepath: std::path::PathBuf::from(format!("{}.py", name)),
            blocks: vec![],
        }
    }

    pub fn push(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn extend(mut self, blocks: impl Iterator<Item=Block>) -> Self {
        self.blocks.extend(blocks);
        self
    }
}

impl codemaker::OutputFile for Module {
    fn path(&self) -> &std::path::Path {
        self.filepath.as_path()
    }
    fn write_into<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for block in &self.blocks {
            block.write_into(writer)?
        }
        Ok(())
    }
}

pub struct Block {
    lines: Vec<String>,
}


impl Block {
    pub fn new() -> Self {
        Block { lines: vec![] }
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
