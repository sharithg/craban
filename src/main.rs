use petgraph::Graph;
use std::collections::VecDeque;
use std::fs::{self};
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
    relative_path: String,
}

fn main() {
    let mut graph: Graph<TsFile, String> = Graph::new();

    // let n1 = graph.add_node(10);
    // let n2 = graph.add_node(5);

    // let _l2 = graph.add_edge(n1, n2, 4);

    let dir = Path::new("./assets/proxysite-cloud-functions/functions/src");

    if let Some(files) = visit_dirs(dir) {
        println!("{:#?}", files);
    } else {
        eprintln!("ERROR visiting directories");
    }
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path) -> Option<Vec<TsFile>> {
    let mut dir_queue = VecDeque::new();
    dir_queue.push_back(dir.to_path_buf());
    let mut ts_files: Vec<TsFile> = Vec::new();

    while let Some(dir) = dir_queue.pop_front() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();
            let path = entry.path();

            if file_type.is_file() {
                // Do something with the file, e.g. print its path
                println!("{}", path.display());
                if let Some(ts_file) = find_imported_files(&entry.path()) {
                    ts_files.push(ts_file);
                } else {
                    println!(
                        "ERROR reading filepath: {}",
                        path.as_os_str().to_str().unwrap()
                    );
                }
            } else if file_type.is_dir() {
                dir_queue.push_back(path);
            }
        }
    }

    Some(ts_files)
}

fn find_imported_files(f_path: &PathBuf) -> Option<TsFile> {
    let mut imports: Vec<TsImport> = Vec::new();

    let file_name = f_path.file_name()?.to_str()?;
    let data = fs::read_to_string(f_path).expect("Unable to read file");
    let import_lines = get_import_lines(&data);

    for line in import_lines {
        if let Some(imp) = parse_import(line.to_owned()) {
            // use origin
            imports.push(imp);
        } else {
            println!(
                "ERROR reading line: {}",
                f_path.to_owned().as_os_str().to_str().unwrap()
            );
        }
    }

    Some(TsFile {
        file_name: file_name.to_string(),
        imports,
        relative_path: String::from(f_path.to_str().unwrap()),
    })

    // let mut imports = Vec::new();
}

fn get_import_lines(data: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let mut inside_import = false;
    let mut import_start_index = 0;
    let mut braces_count = 0;
    // clean special chars like: ×
    let data_cleaned = data
        .replace(|c: char| !c.is_ascii(), "")
        .as_str()
        .to_owned();
    let mut inside_comment = false;
    // let _ = String::from_utf8(Vec::from(data.as_bytes())).expect("Found invalid UTF-8");

    for (i, c) in data_cleaned.chars().enumerate() {
        if data_cleaned[i..].starts_with("//") {
            inside_comment = true;
        } else if inside_comment {
            if c == '\n' {
                inside_comment = false;
            }
        } else if !inside_import && data_cleaned[i..].starts_with("import ") {
            inside_import = true;
            import_start_index = i;
            braces_count = 0;
        } else if inside_import {
            if c == '{' {
                braces_count += 1;
            } else if c == '}' {
                braces_count -= 1;
            }

            if braces_count == 0 && c == '\n' {
                inside_import = false;
                let import_str = &data[import_start_index..=i];
                imports.push(import_str.trim().replace("\n", ""));
            }
        }
    }

    if inside_import {
        let import_str = &data[import_start_index..];
        imports.push(import_str.trim().replace("\n", ""));
    }

    imports
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
