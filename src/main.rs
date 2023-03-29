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

    let files = visit_dirs(dir).expect(&format!("ERROR reading directory: {}", dir.display()));

    // create a hashmap of the file path to a graph node
    for file in files.clone() {
        let path_clone = get_base_project_path(dir, &Path::new(&file.relative_path.clone()));

        let g_node = graph.add_node(path_clone.clone());

        path_to_ts_file.insert(path_clone, g_node);
    }

    for visiting_file in files.clone() {
        let visting_file_relative_path = Path::new(visiting_file.relative_path.as_str());
        let visiting_file_node_key =
            get_base_project_path(dir, &Path::new(&visiting_file.relative_path.clone()));

        // for each file visit its dependancies (imports) and populate the graph
        for import in visiting_file.imports {
            let import_path = Path::new(import.source.as_str());
            let import_abs_path = abs_path_from_dir_to_file(
                visting_file_relative_path.parent().unwrap(),
                &import_path,
            );
            let import_base_path =
                get_base_project_path(dir, &Path::new(&import_abs_path.to_str().unwrap()));

            // get currently visiting file node
            if let Some(import_node) = path_to_ts_file.get(&visiting_file_node_key) {
                // get currently visiting files import node
                if let Some(visiting_file_dependancy_node) =
                    path_to_ts_file.get(import_base_path.as_str())
                {
                    graph.update_edge(
                        *import_node,
                        *visiting_file_dependancy_node,
                        String::from(import_abs_path.to_str().unwrap()),
                    );
                }
            }
        }
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

/**
 * Takes in two paths original_path and relative_path and returns the absolute path of
 * relative_path.
 * Ex:
 * original_path: /Users/linus/src/utils/test
 * relative_path: ../util
 *
 * Returns: /Users/linus/src/utils
 *
 * Since the file util was imported relatively from /Users/linus/src/utils
 */
fn abs_path_from_dir_to_file(original_path: &Path, relative_path: &Path) -> PathBuf {
    let path = Path::new(original_path);
    let parent = path;

    let joined_path = parent.join(relative_path);
    let p = Path::new(joined_path.as_path());

    return PathBuf::from(p.absolutize().unwrap().to_str().unwrap());
}

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
                //
                let ext = path.extension().expect(&format!(
                    "ERROR reading directory: {}",
                    path.as_os_str().to_str().unwrap()
                ));

                // we only support ts files
                if ext == "ts" {
                    let ts_file = find_imported_files(&entry.path()).expect(&format!(
                        "ERROR reading filepath: {}",
                        path.as_os_str().to_str().unwrap()
                    ));
                    ts_files.push(ts_file);
                }
            } else if file_type.is_dir() {
                dir_queue.push_back(path);
            }
        }
    }

    Some(ts_files)
}
