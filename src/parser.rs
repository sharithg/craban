use crate::structs::{TsFile, TsImport, TsImportSource};
use std::{
    fs::{self, canonicalize},
    path::PathBuf,
};

pub fn find_imported_files(f_path: &PathBuf) -> Option<TsFile> {
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
    let mut parts_collection_str: Vec<String> = parts_collection
        .iter()
        .map(|&x| {
            x.trim_end_matches(";")
                .replace("'", "")
                .replace('"', "")
                .into()
        })
        .collect();

    if parts_collection.len() == 2 {
        parts_collection_str.drain(0..1);
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

// test get_import_lines
#[test]
fn test_get_import_lines() {
    let res = get_import_lines(
        "
    import {
        Component
        } from '@angular2/core';
    import defaultMember from 'module-name';
    import   *    as name from 'module-name  ';
    import   {  member }   from '  module-name';
    // import my nice package
    import { member as alias } from 'module-name';

    const val = 'dummy';

    const myFunc = function(){};
    ",
    );
    assert_eq!(res.len(), 5);
}

// test tokenize_import
#[test]
fn test_tokenize_default_import() {
    let import_str = "import * as myModule from 'mylib';";
    let tokens = tokenize_import(import_str);
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

#[test]
fn test_tokenize_whole_module() {
    let import_str = "import 'mylib';";
    let tokens = tokenize_import(import_str);
    assert_eq!(tokens, vec!["mylib"]);
}
