#[test]
fn test_run_add() {
    // Run a new process for cargo run examples/hello.umo
    let output = std::process::Command::new("cargo")
        .args(&["run", "examples/sum.umo"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"45\n");
}
