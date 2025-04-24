use crate::Result;
use std::path::Path;

use std::process::Command;

/// Tries to format the source code of a given file.
pub(crate) fn try_format(path_to_file: &Path) -> Result<()> {
    Command::new("rustfmt")
        .args([path_to_file])
        .status()
        .map(|_| ())
        .map_err(|e| std::io::Error::new(e.kind(), format!("Failed to format file: {}", e)).into())
}
