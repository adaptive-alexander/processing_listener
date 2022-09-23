use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;

/// # read_dir
/// Reads a folder looking for files
///
/// # args:
/// *`dir` - Path to the directory to search
///
/// # returns:
/// A Vec with path buffers.
fn read_input_dir(dir: &Path) -> Vec<PathBuf> {
    // Create Vec for return
    let mut ret_paths: Vec<PathBuf> = Vec::new();

    // Create iterator on directory
    let it = fs::read_dir(dir).unwrap();

    // Iterate entries returning files
    for entry in it {
        if entry.as_ref().unwrap().path().is_dir() == false {
            ret_paths.push(entry.unwrap().path());
        }
    }
    ret_paths
}

/// # dir_setup
/// Checking for proper dirs structure. Creates dirs if missing.
///
/// # args:
/// *`inp_path_string` - Input path string
/// *`out_path_string` - Output path string
pub fn dir_setup(inp_path_string: &str, out_path_string: &str, move_path_string: &str) {
    let mut dirs: Vec<PathBuf> = Vec::new();

    // Create iterator on directory
    let dir_it = fs::read_dir(PathBuf::from(r".\".to_string())).unwrap();

    // Iterate entries
    for entry in dir_it {
        if entry.as_ref().unwrap().path().is_dir() == true {
            dirs.push(entry.unwrap().path())
        }
    }
    match &dirs.iter().find(|&x| {
        *x.file_name().unwrap()
            == *OsStr::new(inp_path_string.replace(&['\\', '.'][..], "").as_str())
    }) {
        Some(_) => {}
        None => {
            println!("Creating input folder");
            fs::create_dir(PathBuf::from(inp_path_string)).unwrap();
            println!("Creating processed folder");
            fs::create_dir(PathBuf::from(move_path_string)).unwrap();
        }
    };
    match &dirs.iter().find(|&x| {
        *x.file_name().unwrap()
            == *OsStr::new(out_path_string.replace(&['\\', '.'][..], "").as_str())
    }) {
        Some(_) => {}
        None => {
            println!("Creating output folder");
            fs::create_dir(PathBuf::from(out_path_string)).unwrap();
        }
    };
}

/// # dir_listener
/// Driver code. First does dir setup, then checks dirs for new files to process.
///
/// # args:
/// *`run_func` - Function to run on files in input folder.
/// *`sleep_time` - Sleep time between listen loops. 10ms is a good starting point.
/// *`inp_path_string` - Path to input files.
/// *`out_path_string` - Path to output files.
/// *`move_path_string` - Path to move processed input files to. Contained within out_path.
pub async fn dir_listener(
    call_back: &dyn Fn(PathBuf, PathBuf, PathBuf),
    sleep_time: &u64,
    inp_path_string: &str,
    out_path_string: &str,
    move_path_string: &str,
) {
    dir_setup(inp_path_string, out_path_string, move_path_string);
    loop {
        // read new files
        let new_file_paths = read_input_dir(&PathBuf::from(inp_path_string));

        // for each new file
        for file in &new_file_paths {
            let process_file = PathBuf::from(file);
            let mut out_path = PathBuf::new();
            let mut move_path = PathBuf::new();

            out_path.push(Path::new(out_path_string));
            out_path.push(&file.file_name().unwrap());

            move_path.push(Path::new(move_path_string));
            move_path.push(file.file_name().unwrap());

            // process input file
            call_back(process_file, out_path, move_path);
        }
        sleep(Duration::from_millis(*sleep_time)).await;
    }
}
