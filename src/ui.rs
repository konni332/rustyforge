use colored::Colorize;
use std::path::Path;

pub fn output_successfull_compile(file_path: &Path) {
    let verbosity = verbosio::get_verbosity!();
    let path = if verbosity > 1 {
        file_path.to_string_lossy()
    } else {
        file_path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or(file_path.to_string_lossy())
    };
    #[cfg(feature = "term-colors")]
    {
        let msg = format!("{} {}", "Compiled".bold().green(), path);
        println!("{}", msg);
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let msg = format!("{} [{}]", "Compiled", path);
        println!("{}", msg);
    }
}

pub fn output_error_compile(file_path: &Path, error: &[u8]) {
    let err_msg = String::from_utf8_lossy(error);
    let verbosity = verbosio::get_verbosity!();
    let path = if verbosity > 1 {
        file_path.to_string_lossy()
    } else {
        file_path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or(file_path.to_string_lossy())
    };

    #[cfg(feature = "term-colors")]
    {
        let msg = format!("{} {}:\n{}", "Failed".bold().red(), path, err_msg);
        println!("{}", msg);
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let msg = format!("{} {}:\n{}", "Failed", path, err_msg);
        println!("{}", msg);
    }
}
