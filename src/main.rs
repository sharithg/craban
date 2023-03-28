use path_absolutize::*;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::stable_graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::fs::canonicalize;
use std::fs::File;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
enum TsImportSource {
    PACKAGE,
    LOCAL,
}

#[derive(Debug, Clone)]
struct TsImport {
    import_source: TsImportSource,
    source: String,
}

#[derive(Debug, Clone)]
struct TsFile {
    imports: Vec<TsImport>,
    file_name: String,
    relative_path: String,
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

fn main() {
    let mut graph: Graph<String, String> = Graph::new();
    let mut path_to_ts_file: HashMap<String, NodeIndex> = HashMap::new();

    let dir = Path::new("./assets/TypeScript-Node-Starter/src");

    if let Some(files) = visit_dirs(dir) {
        // println!("{:#?}", files);
        for file in files.clone() {
            let path_clone = file
                .relative_path
                .clone()
                .replace("/Users/sharithgodamanna/Desktop/Code/ts-analyzer/assets/TypeScript-Node-Starter/src", "");
            let g_node = graph.add_node(path_clone.clone());

            // println!("Idx: {}", g_node.index());
            path_to_ts_file.insert(path_clone, g_node);
        }

        for file_2 in files.clone() {
            // println!("{}", file_2.relative_path);
            let base_file_rel_path = Path::new(file_2.relative_path.as_str());
            for rel_path in file_2.imports {
                // println!("-----{}", rel_path.source);

                let import_path = Path::new(rel_path.source.as_str());
                let rl_path = relative_path_from_dir_to_file(
                    base_file_rel_path.parent().unwrap(),
                    &import_path,
                );

                let source_node_key =
                    get_base_project_path(dir, &Path::new(&file_2.relative_path.clone()));

                if let Some(source_node) = path_to_ts_file.get(&source_node_key) {
                    let path_key =
                        get_base_project_path(dir, &Path::new(&rl_path.to_str().unwrap()));
                    if let Some(found_node) = path_to_ts_file.get(path_key.as_str()) {
                        graph.update_edge(
                            *source_node,
                            *found_node,
                            String::from(rl_path.to_str().unwrap()),
                        );
                    }
                }
            }
        }
    } else {
        eprintln!("ERROR visiting directories");
    }

    let cfg = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    println!("{:?}", cfg);
    let mut f = File::create("example1.dot").unwrap();
    let output = format!("{}", cfg);

    if let Ok(_) = f.write(&output.as_bytes()) {
        println!("{}", "Wrote output graph");
    } else {
        eprintln!("{}", "Error writing graph")
    }
}

fn get_base_project_path(full_path: &Path, relative_path: &Path) -> String {
    let abs_path = std::fs::canonicalize(full_path).unwrap();
    let abs_path_str = abs_path.as_os_str().to_str().unwrap();

    relative_path.to_str().unwrap().replace(abs_path_str, "")
}

fn relative_path_from_dir_to_file(original_path: &Path, relative_path: &Path) -> PathBuf {
    let path = Path::new(original_path);
    let parent = path;

    let joined_path = parent.join(relative_path);
    let p = Path::new(joined_path.as_path());

    return PathBuf::from(p.absolutize().unwrap().to_str().unwrap());
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
                let ext = path.extension().unwrap().to_str().unwrap();

                if ext == "ts" {
                    if let Some(ts_file) = find_imported_files(&entry.path()) {
                        ts_files.push(ts_file);
                    } else {
                        println!(
                            "ERROR reading filepath: {}",
                            path.as_os_str().to_str().unwrap()
                        );
                    }
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
        relative_path: String::from(canonicalize(f_path).unwrap().to_str().unwrap()),
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

    let mut source_str = source.replace("'", "");
    let len = source_str.len();

    if source_str.ends_with("..") {
        // source_str.push_str("/index")
        source_str.replace_range(len - 2..len, "index");
        println!("Str: {}", source_str);
    }

    if source_str.ends_with(".") {
        source_str.replace_range(len - 1..len, "index");
        println!("Str: {}", source_str);
    }

    source_str.push_str(".ts");

    Some(TsImport {
        import_source: import_type,
        source: source_str,
    })
}

fn tokenize_import(import_str: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();

    let parts = import_str.split(" ");
    let parts_collection: Vec<&str> = parts.collect();
    let parts_collection_str: Vec<String> = parts_collection
        .iter()
        .map(|&x| x.trim_end_matches(";").into())
        .collect();

    if parts_collection.len() == 2 {
        return parts_collection_str;
    }

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
                    let rep = parts[i].replace("'", "").replace('"', "");
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
