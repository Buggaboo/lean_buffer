use glob::glob;
use quote::ToTokens;
use std::{collections::HashSet, fs, path::PathBuf};

type SynExternUseVecTuple = (Vec<syn::Item>, Vec<syn::Item>);

/// Parse all instances of ItemExternCrate and ItemUse, then return the input as String without the parsed items
pub fn strip_extern_and_use(
    input: &str,
    item_output: &mut SynExternUseVecTuple,
    output: &mut String,
) -> Result<(), Box<syn::Error>> {
    let ast = syn::parse_file(input)?;

    for item in ast.items {
        match item {
            syn::Item::ExternCrate(_) => {
                item_output.0.push(item);
            }
            syn::Item::Use(_) => {
                item_output.1.push(item);
            }
            _ => {
                let item_string = item.to_token_stream().to_string();
                output.push_str(&item_string);
            }
        }
    }

    Ok(())
}

/// Dedup externs and uses
fn remove_string_duplicates(strings: &Vec<String>) -> Vec<String> {
    let mut unique_strings = HashSet::new();

    for string in strings {
        unique_strings.insert(string.as_str());
    }

    unique_strings.iter().map(|s| s.to_string()).collect()
}

pub fn merge_extern_and_use(
    item_input: &SynExternUseVecTuple,
    out: &mut String,
) -> Result<(), Box<syn::Error>> {
    let externs = remove_string_duplicates(
        &item_input
            .0
            .iter()
            .map(|t| t.to_token_stream().to_string())
            .collect(),
    );
    let uses = remove_string_duplicates(
        &item_input
            .1
            .iter()
            .map(|t| t.to_token_stream().to_string())
            .collect(),
    );

    out.push_str(externs.join("\n").as_str());
    out.push_str(uses.join("\n").as_str());

    Ok(())
}

pub fn glob_generated(path: &PathBuf, suffix_pattern: &str) -> Vec<PathBuf> {
    let glob_path = format!(
        "{}/{}",
        path.to_str().expect("Bad glob pattern"),
        suffix_pattern
    );
    let mut pbs: Vec<PathBuf> = Vec::new();
    for entry in glob(&glob_path).expect("Failed to read glob suffix pattern") {
        pbs.push(entry.expect("GlobError"));
    }
    pbs
}

pub fn glob_and_merge_generated_files(out_path: &PathBuf, suffix_pattern: &str) -> String {
    let file_paths = glob_generated(out_path, suffix_pattern);

    merge_files(&file_paths)
}

pub fn merge_files(file_paths: &Vec<PathBuf>) -> String {
    let mut item_output: SynExternUseVecTuple = (Vec::new(), Vec::new());
    let mut output_discarded = String::new();
    let mut out = String::new();

    for f in file_paths {
        let input = fs::read_to_string(f.as_path()).expect("Error reading file");
        strip_extern_and_use(&input, &mut item_output, &mut output_discarded)
            .expect("Error parsing file");
    }

    merge_extern_and_use(&mut item_output, &mut out)
        .expect("Failed to make externs and uses unique");

    out.push_str(output_discarded.as_str());

    out
}

pub fn generate_pretty_plain_text(utf: &str) -> String {
    let syntax_tree = match syn::parse_file(utf) {
        Ok(parsed) => parsed,
        Err(error) => panic!(
            "There is a problem with parsing the generated rust code: {}",
            error
        ),
    };

    // it seems that genco's code formatting is broken on stable
    prettyplease::unparse(&syntax_tree)
}