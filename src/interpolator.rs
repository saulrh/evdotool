use crate::time_util::Time;

#[derive(Debug)]
pub struct Interpolator {
    /// current time derivative of process variable
    v: f64,
    /// dither error to handle fractional movements
    err: f64,
    /// time of last movement so we can do ds/dt
    last_t: Time,
}

impl Interpolator {
    pub fn new(now: Time) -> Self {
        Self {
            v: 0.0,
            err: 0.0,
            last_t: now,
        }
    }

    pub fn update(&mut self, _time: Time, value: f64) {
        self.v = value;
    }

    pub fn interpolate(&mut self, time: Time) -> i64 {
        // compute how far we want to move on this tick
        let dt = f64::from(time - self.last_t);
        let ds = self.v * dt;

        // our total desired movement is our accumulated error plus
        // this tick's movement
        let err = self.err + ds;

        // we output the integer part and accumulate the fractional
        // part
        let output = err.trunc() as i64;
        self.err = err.fract();

        // dbg![ds, err, self.err];

        // Update tick time and return
        self.last_t = time;
        output as i64
    }
}
