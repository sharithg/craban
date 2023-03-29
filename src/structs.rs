use std::fmt;

#[derive(Debug, Clone)]
pub enum TsImportSource {
    PACKAGE,
    LOCAL,
}

#[derive(Debug, Clone)]
pub struct TsImport {
    pub import_source: TsImportSource,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct TsFile {
    pub imports: Vec<TsImport>,
    pub file_name: String,
    pub relative_path: String,
}

impl fmt::Display for TsFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File name: {}", self.file_name)?;
        writeln!(f, "Relative path: {}", self.relative_path)?;
        writeln!(f, "Imports:")?;
        for import in &self.imports {
            write!(f, "    ")?;
            match import.import_source {
                TsImportSource::PACKAGE => write!(f, "from package ")?,
                TsImportSource::LOCAL => write!(f, "from local file ")?,
            }
            writeln!(f, "{}", import.source)?;
        }
        Ok(())
    }
}
