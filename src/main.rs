use petgraph::Graph;
use std::fs::{self, File};
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum TsImportSource {
    PACKAGE,
    LOCAL,
}

#[derive(Debug)]
struct TsImport {
    import_source: TsImportSource,
    source: String,
}

#[derive(Debug)]
struct TsFile {
    imports: Vec<TsImport>,
    file_name: String,
}

fn main() {
    let mut graph = Graph::new();

    let n1 = graph.add_node(10);
    let n2 = graph.add_node(5);

    let _l2 = graph.add_edge(n1, n2, 4);

    let dir = Path::new("./proxysite-cloud-functions/functions/src");

    visit_dirs(dir);

    // let res = tokenize_import("import { myFunc, myVar } from './myfile';");

    // println!("{:#?}", res);
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path)?;
            } else {
                if let Some(ts_file) = find_imported_files(&entry.path()) {
                    // use origin
                    // println!("{:#?}", ts_file);
                } else {
                    println!(
                        "ERROR reading filepath: {}",
                        path.as_os_str().to_str().unwrap()
                    );
                }
            }
        }
    }
    Ok(())
}

fn find_imported_files(f_path: &PathBuf) -> Option<TsFile> {
    let file = File::open(f_path).unwrap();

    // imports: Vec<TsImport>,
    // file_name: String,
    // exports: Vec<String>,

    let mut imports: Vec<TsImport> = Vec::new();

    let file_name = f_path.file_name()?.to_str()?;

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let file_line = line.unwrap();
        if file_line.starts_with("import") {
            if let Some(imp) = parse_import(file_line.to_owned()) {
                // use origin
                imports.push(imp);
            } else {
                println!(
                    "ERROR reading line: {}",
                    f_path.to_owned().as_os_str().to_str().unwrap()
                );
            }
        }
    }

    Some(TsFile {
        file_name: file_name.to_string(),
        imports,
    })

    // let mut imports = Vec::new();
}

// basic import parser
fn parse_import(line: String) -> Option<TsImport> {
    let mut tokenized_import = tokenize_import(line.as_str());
    tokenized_import.reverse();
    let mut import_type = TsImportSource::PACKAGE;

    if tokenized_import.len() == 0 {
        println!("ERROR: tokenized length 0 for: {}", line.as_str());
        return None;
    }

    if tokenized_import[0].starts_with(".") {
        import_type = TsImportSource::LOCAL
    }

    let source = tokenized_import[0].to_owned();

    Some(TsImport {
        import_source: import_type,
        source: source,
    })
}

fn tokenize_import(import_str: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();

    // Remove leading "import " keyword and trailing semicolon (if any)
    let import_str = import_str
        .trim_end()
        .trim_start_matches("import ")
        .trim_end_matches(";");

    // Split import string by whitespace
    let parts: Vec<&str> = import_str.split_whitespace().collect();

    // Loop through parts and tokenize
    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            // Handle default import syntax
            "*" => {
                tokens.push("*".to_owned());
                i += 1;
            }
            // Handle named import syntax
            "{" => {
                i += 1;
                while i < parts.len() && parts[i] != "}" {
                    let rep = parts[i].replace(",", "");
                    tokens.push(rep.to_owned());

                    i += 1;
                }
                i += 1;
            }
            // Handle from keyword and module path
            "from" => {
                i += 1;
                if i < parts.len() {
                    let rep = parts[i].replace("'", "");
                    tokens.push(rep.to_owned());
                    break;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    tokens
}

#[test]
fn test_tokenize_default_import() {
    let import_str = "import * as myModule from 'mylib';";
    let tokens = tokenize_import(import_str);
    println!("{:#?}", tokens);
    assert_eq!(tokens, vec!["*", "mylib"]);
}

#[test]
fn test_tokenize_named_import() {
    let import_str = "import { myFunc } from './myfile';";
    let tokens = tokenize_import(import_str);
    assert_eq!(tokens, vec!["myFunc", "./myfile"]);
}

#[test]
fn test_tokenize_multiple_named_imports() {
    let import_str = "import { myFunc, myVar } from './myfile';";
    let tokens = tokenize_import(import_str);
    assert_eq!(tokens, vec!["myFunc", "myVar", "./myfile"]);
}

#[test]
fn test_tokenize_relative_path_import() {
    let import_str = "import { myFunc } from '../mydir/myfile';";
    let tokens = tokenize_import(import_str);
    assert_eq!(tokens, vec!["myFunc", "../mydir/myfile"]);
}

#[test]
fn test_tokenize_import_with_trailing_semicolon() {
    let import_str = "import * as myModule from 'mylib';";
    let tokens = tokenize_import(import_str);
    assert_eq!(tokens, vec!["*", "mylib"]);
}

#[test]
fn test_tokenize_import_with_leading_and_trailing_spaces() {
    let import_str = "   import { myFunc } from './myfile';   ";
    let tokens = tokenize_import(import_str);
    assert_eq!(tokens, vec!["myFunc", "./myfile"]);
}

#[test]
fn test_tokenize_invalid_import_syntax() {
    let import_str = "import myFunc from 'mylib';";
    let tokens = tokenize_import(import_str);
    assert_eq!(tokens, vec!["mylib"]);
}
