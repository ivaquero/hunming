use hunming::completion::generate_completions;
use hunming::install::InitShell;

#[test]
fn generate_bash_completions() {
    let output = generate_completions(InitShell::Bash).expect("bash completions should render");

    assert!(output.contains("_hunming"));
    assert!(output.contains("complete"));
}

#[test]
fn generate_zsh_completions() {
    let output = generate_completions(InitShell::Zsh).expect("zsh completions should render");

    assert!(output.contains("compdef") || output.contains("_hunming"));
}

#[test]
fn generate_powershell_completions() {
    let output =
        generate_completions(InitShell::Powershell).expect("powershell completions should render");

    assert!(output.contains("Register-ArgumentCompleter"));
    assert!(output.contains("hunming"));
}
