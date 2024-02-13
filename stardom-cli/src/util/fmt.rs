use std::{fmt, time::Duration};

pub fn efficiency(old: u64, new: u64) -> f32 {
    let old = old as f32;
    let new = new as f32;
    100.0 * (old - new) / old
}

#[derive(Clone, Copy, Debug)]
pub struct FileSize(pub u64);

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const UNITS: [&str; 3] = ["B", "KB", "MB"];
        let bytes = self.0 as f32;
        let i = ((bytes.log10() / 3.0) as usize).min(UNITS.len() - 1);

        write!(f, "{:.1}{}", bytes / 1000.0f32.powi(i as i32), UNITS[i])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Elapsed(pub Duration);

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let secs = self.0.as_secs();
        if secs >= 60 {
            write!(f, "{}m {:02}s", secs / 60, secs % 60)
        } else {
            write!(f, "{}.{:02}s", secs, self.0.subsec_nanos() / 10_000_000)
        }
    }
}
