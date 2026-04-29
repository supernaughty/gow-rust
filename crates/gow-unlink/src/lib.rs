use std::ffi::OsString;
use std::fs;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    // Skip argv[0]
    let operands: Vec<String> = args_vec[1..]
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    if operands.len() != 1 {
        eprintln!(
            "unlink: {}",
            if operands.is_empty() {
                "missing operand".to_string()
            } else {
                format!("extra operand '{}'", operands[1])
            }
        );
        return 2;
    }

    let converted = gow_core::path::try_convert_msys_path(&operands[0]);
    match fs::remove_file(&converted) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("unlink: cannot unlink '{}': {}", converted, e);
            1
        }
    }
}
