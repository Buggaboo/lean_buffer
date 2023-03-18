use std::{env, fs, path::PathBuf};

use lean_buffer_internal::{util::glob_and_merge_generated_files, core::generate_pretty_plain_text};

/// required to activate OUT_DIR in the macro, albeit an empty build.rs
fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Can't locate out_dir"));
    let merged = glob_and_merge_generated_files(&out_dir, "*_lb_gen.rs");
    let dest_dir = out_dir
        .join("merged_gen.lb.rs");
    let out = generate_pretty_plain_text(merged.as_str());
    fs::write(&dest_dir, out).expect("Unable to write merged rs file");
}
