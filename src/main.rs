use path_absolutize::*;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::stable_graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::File;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};
mod cli;
mod parser;
mod structs;
use cli::parse_args;
use parser::find_imported_files;
use structs::TsFile;

fn main() {
    if let Ok(cli_output) = parse_args() {
        if let Some(source_code_dir) = cli_output {
            generate_project_graph(source_code_dir.as_str());
        }
    }
}

fn generate_project_graph(src_dir: &str) {
    let mut graph: Graph<String, String> = Graph::new();
    let mut path_to_ts_file: HashMap<String, NodeIndex> = HashMap::new();

    let dir = Path::new(src_dir);

    if let Some(files) = visit_dirs(dir) {
        // println!("{:#?}", files);
        for file in files.clone() {
            let path_clone = get_base_project_path(dir, &Path::new(&file.relative_path.clone()));

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
                if let Some(ext) = path.extension() {
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
                }
            } else if file_type.is_dir() {
                dir_queue.push_back(path);
            }
        }
    }

    Some(ts_files)
}
