use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use fundsp::hacker::{sine_hz, AudioUnit, BufferArray, BufferRef, NetBackend, U2};
use fundsp::net::{Net, NodeId};
use fundsp::MAX_BUFFER_SIZE;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

const SAMPLES_PER_CHANNEL: usize = 8;
const BUFFER_LEN: usize = 8;
const CHANNELS: usize = 2;
const SAMPLE_RATE: u32 = 48000;
const AUDIO_BUFFER: u32 = 1024;

pub struct SystemBackend {
    buffer: BufferArray<U2>,
    backend: NetBackend,
}

impl SystemBackend {
    pub fn process(&mut self) {
        self.backend.process(
            MAX_BUFFER_SIZE,
            &BufferRef::empty(),
            &mut self.buffer.buffer_mut(),
        );
    }
}

pub struct System {
    root: Net,
    root_id: NodeId,
}

impl System {
    pub fn new() -> Self {
        let sine = sine_hz(440.0) | sine_hz(440.0);
        let mut root = Net::new(0, CHANNELS);
        root.set_sample_rate(SAMPLE_RATE.into());
        let root_id = root.push(Box::new(sine));
        root.pipe_output(root_id);
        root.check();
        System { root, root_id }
    }

    pub fn backend(&mut self) -> SystemBackend {
        let buffer = BufferArray::<U2>::new();
        let backend = self.root.backend();

        SystemBackend { buffer, backend }
    }

    pub fn stop(&mut self) {
        self.root.remove(self.root_id);
        self.root.commit();
    }
}

fn main() {
    static RUN: AtomicBool = AtomicBool::new(true);
    let mut system = System::new();
    let mut backend = system.backend();

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    println!("device: {}", device.name().unwrap());

    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Fixed(AUDIO_BUFFER),
    };
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                assert_no_alloc(|| {
                    process(data, &mut backend);
                });
            },
            move |err| {
                eprintln!("An error occurred on the audio stream: {}", err);
            },
            None,
        )
        .expect("Failed to build output stream");

    ctrlc::set_handler(move || {
        system.stop();
        RUN.store(false, Ordering::SeqCst);
    })
    .unwrap();
    println!("run");
    while RUN.load(Ordering::SeqCst) {
        sleep(Duration::from_secs(1));
    }
    println!("stop");
}

fn process(data: &mut [f32], system: &mut SystemBackend) {
    let count = data.len() / MAX_BUFFER_SIZE / CHANNELS;
    for block in 0..count {
        system.process();
        for wide in 0..BUFFER_LEN {
            let left = system.buffer.at(0, wide);
            let left_ref = left.as_array_ref();
            let right = system.buffer.at(1, wide);
            let right_ref = right.as_array_ref();
            for sample in 0..SAMPLES_PER_CHANNEL {
                let index =
                    block * MAX_BUFFER_SIZE + wide * SAMPLES_PER_CHANNEL + sample;
                let stereo_index = index * 2;
                data[stereo_index] = left_ref[sample];
                data[stereo_index + 1] = right_ref[sample];
            }
        }
    }
}
