use mixed_signals::types::{SignalOrFloat, SignalSpec};

#[test]
fn signal_or_float_static_evaluate_ok() {
    let param = SignalOrFloat::Static(3.5);
    let value = param.evaluate_simple(0.0).unwrap();
    assert_eq!(value, 3.5);
}

#[test]
fn signal_or_float_signal_evaluate_ok() {
    // Constant no longer clamps - returns raw value
    let param = SignalOrFloat::from(SignalSpec::Constant { value: 7.5 });
    let value = param.evaluate_simple(0.0).unwrap();
    assert_eq!(value, 7.5);
}

#[test]
fn signal_or_float_invalid_signal_errs() {
    let param = SignalOrFloat::from(SignalSpec::GaussianNoise {
        seed: 1,
        std_dev: -1.0, // Invalid: negative std_dev
        amplitude: 1.0,
        offset: 0.0,
    });
    assert!(param.evaluate_simple(0.0).is_err());
}
