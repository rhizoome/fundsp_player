use std::process;

use fundsp::biquad_bank::{BiquadBank, BiquadCoefsBank};
use fundsp::hacker::*;
use fundsp::net::Net;
use wide::f32x8;

use crate::runner::SAMPLE_RATE;

// BANK CURRENT

fn build_bank_current() -> impl AudioUnit {
    Net::wrap(Box::new(
        (noise()
            >> split()
            >> (resonator_hz(440.0, 50.0)
                | resonator_hz(440.0 * 2.0, 50.0)
                | resonator_hz(440.0 * 3.0, 50.0)
                | resonator_hz(440.0 * 4.0, 50.0)
                | resonator_hz(440.0 * 5.0, 50.0)
                | resonator_hz(440.0 * 6.0, 50.0)
                | resonator_hz(440.0 * 7.0, 50.0)
                | resonator_hz(440.0 * 8.0, 50.0))
            >> join())
            * 0.1,
    ))
}

// BANK SIMD

fn res_bank(hz: f32) -> An<BiquadBank<f32x8, U8>> {
    let whz: [f32; 8] = [
        hz * 1.0,
        hz * 2.0,
        hz * 3.0,
        hz * 4.0,
        hz * 5.0,
        hz * 6.0,
        hz * 7.0,
        hz * 8.0,
    ];
    let bqc = BiquadCoefsBank::<f32x8, U8>::resonator(
        SAMPLE_RATE as f32,
        f32x8::new(whz),
        f32x8::from(50.0),
    );
    let bq = BiquadBank::with_coefs(bqc);
    An(bq)
}

fn build_bank_simd() -> impl AudioUnit {
    Net::wrap(Box::new(
        noise() >> split() >> res_bank(440.0) >> join() * 0.1,
    ))
}

// TEST_HARMONIC_SERIES

fn sine_hz_sync<F: fundsp::Real>(hz: f32) -> An<Pipe<Constant<U1>, Sine<F>>> {
    constant(hz) >> An(Sine::<F>::with_phase(0.0))
}

fn build_harmonic_series() -> impl AudioUnit {
    let mut base = Net::wrap(Box::new(sine_hz_sync::<f64>(440.0)));
    for i in (3..=64).step_by(2) {
        let n = i as f32;
        base = base + (sine_hz_sync::<f64>(440.0 * n) * (1.0 / n));
    }
    base * 0.5
}

pub fn build(name: &str) -> Box<dyn AudioUnit> {
    match name {
        "bank_current" => Box::new(build_bank_current()),
        "bank_simd" => Box::new(build_bank_simd()),
        "harmonic_series" => Box::new(build_harmonic_series()),
        &_ => {
            println!(
                "\nUnknow build, available builds:
- bank_current
- bank_simd
- harmonic_series"
            );
            process::exit(1);
        }
    }
}
