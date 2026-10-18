use super::*;

#[test]
fn annulla_prima_della_registrazione_dei_controlli_impedisce_avvio() {
    let recorder = Recorder::default();
    recorder.cancel_start("current".into());
    let controls = Arc::new(Controls {
        session_id: "current".into(),
        ..Controls::default()
    });
    let registration = recorder.register(&controls, &Settings::default());
    assert_eq!(controls.begin(), Err(AppError::Cancelled));
    drop(registration);
    assert!(!recorder.stop());
    let next = Arc::new(Controls {
        session_id: "next".into(),
        ..Controls::default()
    });
    let _registration = recorder.register(&next, &Settings::default());
    assert_eq!(next.begin(), Ok(()));
}

#[test]
fn annulla_obsoleto_non_ferma_una_sessione_nuova_e_stop_dopo_avvio_salva() {
    let recorder = Recorder::default();
    let controls = Arc::new(Controls {
        session_id: "current".into(),
        ..Controls::default()
    });
    let _registration = recorder.register(&controls, &Settings::default());
    recorder.cancel_start("old".into());
    assert_eq!(controls.begin(), Ok(()));
    recorder.cancel_start("current".into());
    assert!(controls.stop.load(Ordering::Acquire));
    assert!(*controls.started.lock().unwrap());
}
