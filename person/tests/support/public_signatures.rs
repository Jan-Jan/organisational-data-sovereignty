//! The scan of a Rust source's public signatures, shared by the signature
//! tests of the person unit (person/tests/crate_discipline.rs) and the
//! org-members unit (org-members/tests/tree_error_mapping.rs).

/// What a `{` opened, as far as the scan of public signatures needs it.
#[derive(Clone, Copy, PartialEq)]
enum Block {
    /// A `pub enum`'s body, or a struct variant's inside one.
    PublicEnum,
    /// A `pub struct`'s body.
    PublicStruct,
    /// An `impl Trait for Type` body.
    TraitImpl,
    /// Anything else: a function body, an inherent impl, a module.
    Other,
}

/// The public signatures of a Rust source: each `pub fn`, `pub const fn`,
/// `pub type`, `pub use`, `pub struct` or `pub enum` item (free or a method)
/// and each method of a trait impl, from its first line to the line that
/// opens its body or ends it, joined; and each variant of a `pub enum` and
/// each `pub` field of a `pub struct`, one line each. Comment and attribute
/// lines are not read.
pub fn public_signatures(source: &str) -> Vec<String> {
    const ITEM_STARTS: [&str; 6] = ["pub fn ", "pub const fn ", "pub type ", "pub use ", "pub struct ", "pub enum "];
    let mut signatures = Vec::new();
    let mut current: Option<String> = None;
    let mut blocks: Vec<Block> = Vec::new();
    let mut statement = String::new();
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") || line.starts_with("#[") || line.starts_with("#!") {
            continue;
        }
        let inside = blocks.last().copied();
        if current.is_none() {
            let starts_item = ITEM_STARTS.iter().any(|start| line.starts_with(start))
                || (inside == Some(Block::TraitImpl) && line.starts_with("fn "));
            let is_member = !line.starts_with('}')
                && (inside == Some(Block::PublicEnum)
                    || (inside == Some(Block::PublicStruct) && line.starts_with("pub ")));
            if is_member {
                signatures.push(format!("{line} "));
            } else if starts_item {
                current = Some(String::new());
            }
        }
        if let Some(signature) = current.as_mut() {
            signature.push_str(line);
            signature.push(' ');
            if line.contains('{') || line.ends_with(';') {
                signatures.extend(current.take());
            }
        }
        statement.push_str(line);
        statement.push(' ');
        for character in line.chars() {
            match character {
                '{' => blocks.push(if inside == Some(Block::PublicEnum) || statement.starts_with("pub enum ") {
                    Block::PublicEnum
                } else if statement.starts_with("pub struct ") {
                    Block::PublicStruct
                } else if statement.starts_with("impl") && statement.contains(" for ") {
                    Block::TraitImpl
                } else {
                    Block::Other
                }),
                '}' => {
                    blocks.pop();
                }
                _ => {}
            }
        }
        if line.contains(['{', '}', ';']) || line.ends_with(',') {
            statement.clear();
        }
    }
    signatures
}

/// The public signatures of every `.rs` file directly under `source_dir`,
/// each with its file's path. Fails when there is none, so a scan that reads
/// no source cannot pass unread.
pub fn crate_public_signatures(source_dir: &std::path::Path) -> Vec<(std::path::PathBuf, String)> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(source_dir).expect("the crate's src is readable") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("a source file is readable");
        found.extend(public_signatures(&source).into_iter().map(|signature| (path.clone(), signature)));
    }
    assert!(!found.is_empty(), "no public signature found under {}", source_dir.display());
    found
}
