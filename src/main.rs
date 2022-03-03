#![allow(dead_code, unused_variables)]

use std::cmp::Ordering;
use std::io::BufRead;
use std::{io::stdin};
use std::sync::mpsc;
use rustfft::num_complex::Complex32;
use sdl2::audio::{AudioSpecDesired, AudioCallback, AudioSpec};

const SAMPLE_SIZE: usize = 4096;
const RATIO_SHOWN: usize = 32;
const DISP_NUM: usize = 4096/64;
const COMMA: f64 = 1.0594630943592953;
static NOTES: &'static [(f64, &'static str)] = &[
    (220.0, "La"),
    (233.082, "La#"),
    (246.942, "Si"),
    (261.626, "Do"),
    (277.183, "Do#"),
    (293.665, "Ré"),
    (311.127, "Ré#"),
    (329.628, "Mi"),
    (349.228, "Fa"),
    (369.994, "Fa#"),
    (391.995, "Sol"),
    (415.305, "Sol#"),
    (440.0, "La"),
];

fn main() {
    let sdl = sdl2::init().unwrap();
    let audio = sdl.audio().unwrap();
    let spec = AudioSpecDesired {
        channels: Some(1),
        freq: None,
        samples: Some(SAMPLE_SIZE as u16),
    };

    let (tx, rx) = mpsc::channel();

    let cap = audio.open_capture(
        None, 
        &spec, 
        |spec| {
            println!("Using config: {:?}", &spec);
            Note { 
                spec, 
                tx,
                fft: rustfft::FftPlanner::<f32>::new(),
            }
        },
    ).unwrap();
    cap.resume();

    let t = std::thread::spawn(move || {
        loop { 
            let msg = rx.recv().unwrap();
            soundboard(&msg);
            display_note(&msg);
        }
    });

    let mut s = String::new();
    let stdin = stdin();
    stdin.lock().read_line(&mut s).unwrap();
}

struct Note {
    spec: AudioSpec,
    tx: mpsc::Sender<Msg>,
    fft: rustfft::FftPlanner<f32>,
}

#[derive(Debug)]
struct Msg {
    freq: Vec<Complex32>,
    samples_per_second: i32,
}

impl AudioCallback for Note {
    type Channel = f32;

    fn callback(&mut self, samples: &mut [Self::Channel]) {
        let fft = self.fft.plan_fft_forward(samples.len());
        let mut buf: Vec<Complex32> = samples.iter().map(|&f| Complex32::new(f, 0.0)).collect();
        fft.process(&mut buf);

        let _osef = self.tx.send(Msg { freq: buf, samples_per_second: self.spec.freq });
    }
}

fn normalize_freqs(freqs: &mut[Complex32]) {
    //let total_energy: f32 = freqs.iter().map(|c| c.norm()).sum();
    let largest_energy = *freqs.iter()
        .max_by(|l, r| l.norm()
            .partial_cmp(&r.norm())
            .unwrap_or(Ordering::Less)
        ).unwrap();
    
    for f in freqs.iter_mut() {
        *f /= largest_energy;
    }
}

fn compute_frequency(bin_index: usize, samples_per_second: usize, sample_count: usize) -> f64 {
    bin_index as f64 * samples_per_second as f64 / sample_count as f64 // lol rip accuracy
}

fn hz_to_note(mut hz: f64) -> &'static str {
    let mut iters = 0;
    while hz > 440.0 && iters < 100 { hz /= 2.0; iters += 1; }
    while hz < 220.0 && iters < 100 { hz *= 2.0; iters += 1; }
    if iters >= 100 { return "?" }

    let idx = NOTES.partition_point(|(x, _)| *x < hz);
    let dd = (hz - NOTES.get(idx - 1).map(|x| x.0).unwrap_or(100000.0)).abs();
    let du = (hz - NOTES.get(idx    ).map(|x| x.0).unwrap_or(100000.0)).abs();
    if dd < du {
        NOTES[idx - 1].1
    } else {
        NOTES[idx].1
    }
}

fn soundboard(msg: &Msg) {
    use std::fmt::Write;

    // The low frequencies are very present because i don't do windowing to clean up the signal.
    let mut s = String::new();
    let mut freqs = msg.freq[0..SAMPLE_SIZE / RATIO_SHOWN].to_vec(); //&msg.freq[0..msg.freq.len() / 2];
    let sample_count = msg.freq.len();
    let freq_num = freqs.len();
    normalize_freqs(&mut freqs);

    let chunk_size = freq_num / DISP_NUM;
    for (i, c) in freqs.chunks(chunk_size).enumerate() {
        s.clear();
        let hz = compute_frequency(i * chunk_size, msg.samples_per_second as usize, sample_count);
        write!(s, "{:>5.0} {:>4}: ", hz, hz_to_note(hz)).unwrap();
        let avg: f32 = c.iter().map(|x| x.norm()).sum::<f32>() / c.len() as f32;
        for _ in 0..(avg * 40.0).round() as i32 {
            s.push('|');
        }
        println!("{}", &s);
    }
    println!();
}

fn display_note(msg: &Msg) {
    // beware! if you take freq[n..] where n > 0 all indicies are offset!
    let interesting = &msg.freq[0..SAMPLE_SIZE/RATIO_SHOWN];
    let max_idx = interesting.iter()
        .enumerate()
        .fold((0, 0.0), |(maxi, maxx), (i, x)| {
            let norm = x.norm_sqr(); 
            if norm > maxx { (i, norm) } else { (maxi, maxx) }
    });
    let hz = compute_frequency(max_idx.0, msg.samples_per_second as usize, msg.freq.len());
    println!("{:>5} {:>4}", hz, hz_to_note(hz));
}
