use std::process;

use fundsp::biquad_bank::{BiquadBank, BiquadCoefsBank};
use fundsp::hacker::*;
use wide::{f32x8, f64x4};

use crate::runner::SAMPLE_RATE;

// BUTTER BANK

fn butter_bank(hz: f64) -> An<BiquadBank<f64x4, U4>> {
    let whz: [f64; 4] = [hz * 1.0, hz * 2.0, hz * 3.0, hz * 4.0];
    let bqc = BiquadCoefsBank::<f64x4, U4>::butter_lowpass(
        SAMPLE_RATE as f32,
        f64x4::new(whz),
    );
    let bq = BiquadBank::with_coefs(bqc);
    An(bq)
}

fn build_butter_bank() -> impl AudioUnit {
    (noise() >> split() >> butter_bank(440.0) >> (sink() | sink() | sink() | pass()))
        * 0.1
}

// BANK CURRENT

fn build_bank_current() -> impl AudioUnit {
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
        * 0.1
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
    (noise() >> split() >> res_bank(440.0) >> join()) * 0.1
}

// TEST_HARMONIC_SERIES

fn sine_hz_sync<F: fundsp::Real>(hz: f32) -> An<Pipe<Constant<U1>, Sine<F>>> {
    constant(hz) >> An(Sine::<F>::with_phase(0.0))
}

fn build_harmonic_series() -> impl AudioUnit {
    busi::<U8, _, _>(|i| {
        (0.5 / (i * 2 + 1) as f32) * sine_hz_sync::<f32>(440.0 * (i * 2 + 1) as f32)
    })
}

pub fn build(name: &str) -> Box<dyn AudioUnit> {
    match name {
        "bank_butter" => Box::new(build_butter_bank()),
        "bank_current" => Box::new(build_bank_current()),
        "bank_simd" => Box::new(build_bank_simd()),
        "harmonic_series" => Box::new(build_harmonic_series()),
        &_ => {
            println!(
                "\nUnknow build, available builds:
- bank_butter
- bank_current
- bank_simd
- harmonic_series"
            );
            process::exit(1);
        }
    }
}
