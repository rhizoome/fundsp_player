use std::ptr;
use std::thread::sleep;
use std::time::Duration;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait};

pub const SAMPLES_PER_CHANNEL: usize = 8;
pub const BUFFER_LEN: usize = 8;
pub const CHANNELS: usize = 2;
pub const SAMPLE_RATE: u32 = 48000;
pub const AUDIO_BUFFER: u32 = 1024;
use fundsp::audiounit::AudioUnit;
use fundsp::hacker::{BufferArray, BufferRef, Net, NetBackend, U2};
//use fundsp::net::NodeId;
use fundsp::MAX_BUFFER_SIZE;

use crate::build::build;

pub struct RunnerBackend {
    buffer: BufferArray<U2>,
    backend: NetBackend,
}

impl RunnerBackend {
    pub fn process(&mut self) {
        self.backend.process(
            MAX_BUFFER_SIZE,
            &BufferRef::empty(),
            &mut self.buffer.buffer_mut(),
        );
    }
}

pub struct Runner {
    root: Net,
    //root_id: NodeId,
}

impl Runner {
    pub fn new(build_name: &str) -> Self {
        let mut root = Net::new(0, CHANNELS);
        root.set_sample_rate(SAMPLE_RATE.into());
        let graph = build(build_name);
        let outputs = graph.outputs().clone();
        let root_id = root.push(graph);
        if outputs == 2 {
            root.pipe_output(root_id);
        } else {
            root.connect_output(root_id, 0, 0);
            root.connect_output(root_id, 0, 1);
        }
        root.check();
        Runner { root } // ,root_id }
    }

    pub fn backend(&mut self) -> RunnerBackend {
        let buffer = BufferArray::<U2>::new();
        let backend = self.root.backend();

        RunnerBackend { buffer, backend }
    }

    // TODO: add some interfaction with the process
    //pub fn stop(&mut self) {
    //    self.root.remove(self.root_id);
    //    self.root.commit();
    //}
}

pub fn dummy(seconds: u32, build_name: &str) {
    let sink: [f32; 8] = [0.0; 8];
    let sink_ptr = sink.as_ptr() as *mut [f32; 8];
    let mut runner = Runner::new(build_name);
    let mut backend = runner.backend();
    let count: usize = SAMPLE_RATE as usize / MAX_BUFFER_SIZE * 2 * seconds as usize;
    for _ in 0..count {
        backend.process();
        // Avoid optimizations
        for wide in 0..BUFFER_LEN {
            for channel in 0..1 {
                let wide_buf = backend.buffer.at(channel, wide).to_array();
                unsafe {
                    ptr::write_volatile(sink_ptr, wide_buf);
                }
            }
        }
    }
}

pub fn live(device_name: Option<&str>, build_name: &str) {
    let mut runner = Runner::new(build_name);
    let mut backend = runner.backend();

    let host = cpal::default_host();
    let devices = host.devices().expect("Failed to get devices");

    let mut device_found = None;
    if let Some(name) = device_name {
        for device in devices {
            if device.name().unwrap() == name {
                device_found = Some(device);
                break;
            }
        }
    } else {
        device_found = host.default_output_device();
    }
    if let Some(device) = device_found {
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
    } else {
        println!("\nUnknown device, available devices:");
        for device in host.output_devices().unwrap() {
            println!("- {}", device.name().unwrap());
        }
    }
}

fn process(data: &mut [f32], runner: &mut RunnerBackend) {
    let count = data.len() / MAX_BUFFER_SIZE / CHANNELS;
    for block in 0..count {
        runner.process();
        for wide in 0..BUFFER_LEN {
            let left = runner.buffer.at(0, wide);
            let left_ref = left.as_array_ref();
            let right = runner.buffer.at(1, wide);
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
