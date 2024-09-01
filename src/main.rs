use std::thread::sleep;
use std::time::Duration;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait};
use fundsp::hacker::*;
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

pub fn sine_hz_sync(hz: f32) -> An<Pipe<Constant<U1>, Sine>> {
    constant(hz) >> An(Sine::with_phase(0.0))
}

fn build() -> impl AudioUnit {
    let mut base = Net::wrap(Box::new(sine_hz_sync(440.0)));
    for i in (3..=(64 + 32 + 8)).step_by(2) {
        let n = i as f32;
        base = base + (sine_hz_sync(440.0 * n) * (1.0 / n));
    }
    base * 0.5
}

impl System {
    pub fn new() -> Self {
        let mut root = Net::new(0, CHANNELS);
        root.set_sample_rate(SAMPLE_RATE.into());
        let graph = build();
        let outputs = graph.outputs().clone();
        let root_id = root.push(Box::new(graph));
        if outputs == 2 {
            root.pipe_output(root_id);
            println!("huhu");
        } else {
            root.connect_output(root_id, 0, 0);
            root.connect_output(root_id, 0, 1);
        }
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
    let mut system = System::new();
    let mut backend = system.backend();

    let host = cpal::default_host();
    let devices = host.devices().expect("Failed to get devices");

    let device_name = "Loopback Audio";
    let mut desired_device = None;
    for device in devices {
        if device.name().unwrap() == device_name {
            desired_device = Some(device);
            break;
        }
    }
    let device = desired_device.expect("Device not found");

    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Fixed(AUDIO_BUFFER),
    };
    let _stream = device
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

    loop {
        sleep(Duration::from_secs(10));
    }
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
