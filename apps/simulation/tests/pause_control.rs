use speciate::ipc::SimCommand;

#[test]
fn test_set_paused_command_variant_exists() {
    let pause_cmd = SimCommand::SetPaused(true);
    match pause_cmd {
        SimCommand::SetPaused(paused) => assert!(paused),
        _ => panic!("Expected SetPaused variant"),
    }

    let resume_cmd = SimCommand::SetPaused(false);
    match resume_cmd {
        SimCommand::SetPaused(paused) => assert!(!paused),
        _ => panic!("Expected SetPaused variant"),
    }
}

#[cfg(feature = "dev-tools")]
use speciate::ipc::bridge::NapiApp;
#[cfg(feature = "dev-tools")]
use std::sync::Arc;
#[cfg(feature = "dev-tools")]
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
#[cfg(feature = "dev-tools")]
fn test_pause_command_sets_paused_flag() {
    let (tx, rx) = crossbeam_channel::bounded(128);
    let paused = Arc::new(AtomicBool::new(false));

    let mut app = NapiApp::new(rx, 5, ".".to_string(), None);
    app.set_paused_flag(Arc::clone(&paused));

    assert!(!paused.load(Ordering::SeqCst), "Should start unpaused");

    tx.send(SimCommand::SetPaused(true)).expect("Failed to send pause command");
    app.process_commands();

    assert!(paused.load(Ordering::SeqCst), "Should be paused after SetPaused(true)");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_resume_command_clears_paused_flag() {
    let (tx, rx) = crossbeam_channel::bounded(128);
    let paused = Arc::new(AtomicBool::new(false));

    let mut app = NapiApp::new(rx, 5, ".".to_string(), None);
    app.set_paused_flag(Arc::clone(&paused));

    tx.send(SimCommand::SetPaused(true)).expect("Failed to send pause command");
    app.process_commands();
    assert!(paused.load(Ordering::SeqCst), "Should be paused");

    tx.send(SimCommand::SetPaused(false)).expect("Failed to send resume command");
    app.process_commands();

    assert!(!paused.load(Ordering::SeqCst), "Should be unpaused after SetPaused(false)");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_pause_toggle_multiple_times() {
    let (tx, rx) = crossbeam_channel::bounded(128);
    let paused = Arc::new(AtomicBool::new(false));

    let mut app = NapiApp::new(rx, 5, ".".to_string(), None);
    app.set_paused_flag(Arc::clone(&paused));

    for _ in 0..5 {
        tx.send(SimCommand::SetPaused(true)).expect("Failed to send pause");
        app.process_commands();
        assert!(paused.load(Ordering::SeqCst));

        tx.send(SimCommand::SetPaused(false)).expect("Failed to send resume");
        app.process_commands();
        assert!(!paused.load(Ordering::SeqCst));
    }
}
