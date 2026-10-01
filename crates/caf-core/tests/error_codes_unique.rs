//! DoD-2/DoD-3 T2-BOOT-04: tidak ada dua kode error yang sama, dan `docs/error-codes.md`
//! tidak pernah basi karena selalu dibandingkan langsung dengan hasil generate dari kode
//! sumber. Jalankan `UPDATE_GOLDEN=1 cargo test -p caf-core --test error_codes_unique`
//! untuk menulis ulang file dokumentasi setelah menambah/mengubah varian error.

use std::collections::HashSet;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("caf-core diharapkan berada di <root>/crates/caf-core")
        .to_path_buf()
}

#[test]
fn no_duplicate_error_codes() {
    let codes: Vec<&'static str> = caf_core::CatermError::all_known_for_registry()
        .iter()
        .map(caf_core::CatermError::code)
        .collect();
    let unique: HashSet<&'static str> = codes.iter().copied().collect();
    assert_eq!(
        codes.len(),
        unique.len(),
        "kode error terduplikasi: {codes:?}"
    );
}

#[test]
fn error_codes_md_matches_generated_source() {
    let generated = caf_core::CatermError::generate_registry_markdown();
    let doc_path = workspace_root().join("docs").join("error-codes.md");

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(doc_path.parent().expect("docs dir")).expect("mkdir docs/");
        std::fs::write(&doc_path, &generated).expect("gagal menulis docs/error-codes.md");
    }

    let existing = std::fs::read_to_string(&doc_path).unwrap_or_default();
    assert_eq!(
        existing, generated,
        "docs/error-codes.md basi dari src/error.rs — jalankan ulang dengan UPDATE_GOLDEN=1"
    );
}
