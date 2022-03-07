use std::fmt::Display;

use Note::*;

#[derive(Debug, Copy, Clone)]
pub enum Note {
    La,
    LaD,
    Si,
    Do,
    DoD,
    Re,
    ReD,
    Mi,
    Fa,
    FaD,
    Sol,
    SolD,
}

impl Note {
    const HZ: &'static [(Note, f64, &'static str)] = &[
        (La,   220.000, "La"),
        (LaD,  233.082, "La#"),
        (Si,   246.942, "Si"),
        (Do,   261.626, "Do"),
        (DoD,  277.183, "Do#"),
        (Re,   293.665, "Ré"),
        (ReD,  311.127, "Ré#"),
        (Mi,   329.628, "Mi"),
        (Fa,   349.228, "Fa"),
        (FaD,  369.994, "Fa#"),
        (Sol,  391.995, "Sol"),
        (SolD, 415.305, "Sol#"),
    ];

    pub fn from_int(x: i32) -> Option<Note> {
        Self::HZ.get(x as usize).map(|x| x.0)
    }

    pub const fn as_hz(self) -> f64 {
        Self::HZ[self as usize].1
    }

    pub const fn as_str(self) -> &'static str {
        Self::HZ[self as usize].2
    }

    pub fn from_hz(mut hz: f64) -> Option<Self> {
        let mut iters = 0;
        let max_iters = 20;
        while hz >= Self::HZ[SolD as usize].1 && iters < max_iters { hz /= 2.0; iters += 1; }
        while hz <  Self::HZ[La   as usize].1 && iters < max_iters { hz *= 2.0; iters += 1; }
        if iters >= max_iters { return None }
    
        let idx = Self::HZ.partition_point(|(_, x, _)| *x < hz);
        let dd = (hz - Self::HZ.get(idx - 1).map(|x| x.1).unwrap_or(std::f64::INFINITY)).abs();
        let du = (hz - Self::HZ.get(idx    ).map(|x| x.1).unwrap_or(std::f64::INFINITY)).abs();
        if dd < du {
            Self::from_int(idx as i32 - 1)
        } else {
            Self::from_int(idx as i32)
        }
    }

    pub fn hz_to_str(hz: f64) -> &'static str {
        Self::from_hz(hz).map(|x| x.as_str()).unwrap_or("?")
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
