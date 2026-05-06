use hunming::install::{
    bash_managed_block, insert_managed_block, powershell_managed_block, MANAGED_BLOCK_END,
    MANAGED_BLOCK_START,
};

#[test]
fn inserts_block_into_empty_content() {
    let block = bash_managed_block("/tmp/hunming/generated/bash.sh");

    let result = insert_managed_block("", &block);

    assert_eq!(result, block);
}

#[test]
fn appends_block_without_touching_user_content() {
    let original = "export PATH=\"$HOME/bin:$PATH\"\n";
    let block = bash_managed_block("/tmp/hunming/generated/bash.sh");

    let result = insert_managed_block(original, &block);

    assert!(result.starts_with(original));
    assert!(result.contains(MANAGED_BLOCK_START));
    assert!(result.contains(MANAGED_BLOCK_END));
    assert!(result.ends_with(&block));
}

#[test]
fn replaces_existing_block() {
    let original = format!(
        "before\n{start}\nold\n{end}\nafter\n",
        start = MANAGED_BLOCK_START,
        end = MANAGED_BLOCK_END
    );
    let block = powershell_managed_block("/tmp/hunming/generated/powershell.ps1");

    let result = insert_managed_block(&original, &block);

    assert!(result.starts_with("before\n"));
    assert!(result.ends_with("after\n"));
    assert!(result.contains(&block));
    assert!(!result.contains("old"));
}

#[test]
fn powershell_block_matches_expected_shape() {
    let block = powershell_managed_block("/tmp/hunming/generated/powershell.ps1");

    assert_eq!(
        block,
        "# >>> hunming init >>>\n$hunmingProfile = \"/tmp/hunming/generated/powershell.ps1\"\nif (Test-Path $hunmingProfile) {\n    . $hunmingProfile\n}\n# <<< hunming init <<<\n"
    );
}
