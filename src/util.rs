pub fn real_reg_file_without_symlink(pat: &std::path::Path) -> bool {
    !pat.is_symlink() && pat.is_file()
}

pub fn real_dir_without_symlink(pat: &std::path::Path) -> bool {
    !pat.is_symlink() && pat.is_dir()
}
