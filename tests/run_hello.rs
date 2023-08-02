#[test]
fn test_run_hello() {
    // Run a new process for cargo run examples/hello.umo
    let output = std::process::Command::new("cargo")
        .args(&["run", "examples/hello.umo"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"Hello, world!\n");
}
