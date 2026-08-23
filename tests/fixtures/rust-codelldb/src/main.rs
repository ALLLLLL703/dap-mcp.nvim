//! Small deterministic debuggee for the real CodeLLDB acceptance path.

/// Produces a stable local-variable graph for breakpoint inspection.
fn compute(base: i32) -> i32 {
    let doubled = base * 2;
    let answer = doubled + 2;
    answer
}

/// Allows this Linux-only fixture process to be attached under Yama scope 1.
#[cfg(target_os = "linux")]
fn allow_debugger_attach() {
    // SAFETY: prctl is called with the documented PR_SET_PTRACER option and constant argument.
    let result = unsafe { libc::prctl(libc::PR_SET_PTRACER, libc::PR_SET_PTRACER_ANY) };
    assert_eq!(
        result,
        0,
        "failed to allow debugger attach: {}",
        std::io::Error::last_os_error()
    );
}

/// Keeps non-Linux fixture builds portable without changing process policy.
#[cfg(not(target_os = "linux"))]
fn allow_debugger_attach() {}

/// Runs the fixture once and prints its observable result.
fn main() {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|argument| argument == "--wait-for-attach")
    {
        allow_debugger_attach();
        println!("pid={}", std::process::id());
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
    let seed = 20;
    let result = compute(seed);
    if arguments
        .iter()
        .any(|argument| argument == "--pause-target")
    {
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_secs(10) {
            std::hint::spin_loop();
        }
    }
    println!("result={result}");
}
