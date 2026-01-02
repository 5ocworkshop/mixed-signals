use mixed_signals::types::SignalSpec;

#[test]
fn signal_spec_build_ok() {
    let spec = SignalSpec::Sine {
        frequency: 2.0,
        amplitude: 0.5,
        offset: 0.0,
        phase: 0.0,
    };
    let signal = spec.build().unwrap();
    let value = signal.sample(0.125);
    assert!((value - 0.5).abs() < 0.001);
}

#[test]
fn signal_spec_build_invalid_gaussian_errs() {
    let spec = SignalSpec::GaussianNoise {
        seed: 0,
        std_dev: -1.0, // Invalid: negative std_dev
        amplitude: 1.0,
        offset: 0.0,
    };
    assert!(spec.build().is_err());
}

#[test]
fn signal_spec_build_composed_propagates_error() {
    let spec = SignalSpec::Add {
        a: Box::new(SignalSpec::Constant { value: 1.0 }),
        b: Box::new(SignalSpec::GaussianNoise {
            seed: 0,
            std_dev: -1.0, // Invalid: negative std_dev
            amplitude: 1.0,
            offset: 0.0,
        }),
    };
    assert!(spec.build().is_err());
}
