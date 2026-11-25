use std::fs::{read_dir, symlink_metadata};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::{env, println, vec};

#[derive(Clone)]
struct Node {
    name: String,
    size: f64,
}

fn is_ignored_dir(path: &Path) -> bool {
    let ignored_dirs = vec![Path::new("/proc")];

    ignored_dirs.contains(&path)
}

fn search_tree(path: &Path, level: i32, root_dev: u64) -> Option<Node> {
    let metadata = match symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return None,
    };

    if metadata.dev() != root_dev {
        return None;
    }

    if metadata.file_type().is_symlink() {
        return None;
    }

    let mut cur_size: f64 = 0.0;

    let mut children: Vec<Node> = Vec::new();

    let entries = match read_dir(&path) {
        Ok(entry) => entry,
        Err(_) => return None,
    };

    for entry in entries {
        let entry_path = entry.expect("Error getting entry").path();
        if entry_path.is_dir() && !is_ignored_dir(&entry_path) {
            if let Some(childs_node) = search_tree(entry_path.as_path(), level + 1, root_dev) {
                cur_size = cur_size + childs_node.size;
                children.push(childs_node);
            }
        } else if entry_path.is_file() {
            cur_size = cur_size + (entry_path.metadata().unwrap().len() as f64);
        }
    }

    Some(Node {
        name: path.display().to_string(),
        size: cur_size,
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut dirs: Vec<String> = vec![];

    if args.len() == 1 {
        dirs.push(env::current_dir().unwrap().display().to_string());
    } else {
        for arg in args.iter().skip(1) {
            dirs.push(arg.to_owned());
        }
    }

    let exp: u32 = 9;
    let base: u64 = 10;

    for dir in dirs.iter() {
        let root = Path::new(dir);

        let root_metadata = match symlink_metadata(root) {
            Ok(metadata) => metadata,
            Err(_) => return,
        };

        let root_dev = root_metadata.dev();

        let out = match search_tree(root, 0, root_dev) {
            Some(node) => node,
            None => return,
        };

        println!("{}, {:.2}gb", out.name, out.size / (base.pow(exp)) as f64);
    }
}
