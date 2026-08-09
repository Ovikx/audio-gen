/// This file is LLM-generated. LLM was prompted to implement Jezar Wakefield's Freeverb algorithm.
use crate::{
    context::AudioContext,
    source::{NodeOutput, Source},
};

/// Fixed input gain applied before the comb filter bank.
/// Compensates for the 8-way parallel summation to prevent clipping.
const FIXED_GAIN: f32 = 0.015;

/// One-pole smoothing coefficient for `room_size` and `damping`.
/// At 44 100 Hz this gives approximately a 22 ms time constant — fast
/// enough for real-time automation yet slow enough to prevent the
/// discontinuity clicks that occur when these parameters are stepped or
/// rapidly modulated.
const SMOOTH_COEFF: f32 = 0.001;

/// Comb-filter delay lengths in samples at 44 100 Hz.
/// Taken directly from Jezar's original Freeverb source.
const COMB_TUNINGS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];

/// Allpass-filter delay lengths in samples at 44 100 Hz.
const ALLPASS_TUNINGS: [usize; 4] = [556, 441, 341, 225];

// ---------------------------------------------------------------------------
// Internal DSP primitives
// ---------------------------------------------------------------------------

struct CombFilter {
    buf: Vec<f32>,
    index: usize,
    lp_store: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        CombFilter {
            buf: vec![0.0; size],
            index: 0,
            lp_store: 0.0,
        }
    }

    /// Processes one sample through the feedback comb with an embedded
    /// one-pole lowpass in the feedback path.
    ///
    /// - `feedback` — room-size coefficient (0..=1)
    /// - `damp`     — high-frequency damping coefficient (0..=1)
    fn process(&mut self, input: f32, feedback: f32, damp: f32) -> f32 {
        let output = self.buf[self.index];
        self.lp_store = output * (1.0 - damp) + self.lp_store * damp;
        self.buf[self.index] = input + self.lp_store * feedback;
        self.index = (self.index + 1) % self.buf.len();
        output
    }
}

struct AllpassFilter {
    buf: Vec<f32>,
    index: usize,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        AllpassFilter {
            buf: vec![0.0; size],
            index: 0,
        }
    }

    /// Schroeder allpass section with a fixed feedback/feedforward
    /// coefficient of 0.5.
    fn process(&mut self, input: f32) -> f32 {
        let bufout = self.buf[self.index];
        self.buf[self.index] = input + bufout * 0.5;
        self.index = (self.index + 1) % self.buf.len();
        bufout - input
    }
}

// ---------------------------------------------------------------------------
// FreeverbNode
// ---------------------------------------------------------------------------

/// Mono [Freeverb](https://ccrma.stanford.edu/~jos/pasp/Freeverb.html) reverb node.
///
/// # Algorithm
///
/// Freeverb is a classic, computationally efficient algorithmic reverberator
/// designed by Jezar at Dreampoint in 2000. It is composed of two stages:
///
/// ```text
///                    ┌──────────────────────────────────────────┐
///                    │            Comb Filter Bank               │
///  in ──┬─── ×gain ──┤  comb[0]  comb[1] … comb[7]  (parallel) ├──Σ──┐
///       │            └──────────────────────────────────────────┘     │
///       │                                                              ▼
///       │            ┌──────────────────────────────────────────┐     │
///       │            │           Allpass Cascade                │     │
///       │            │  ap[0] → ap[1] → ap[2] → ap[3] (series) │◄────┘
///       │            └──────────────────────────────────────────┘
///       │                              │
///       │         ×dry                 │ ×wet
///       └───────────────────────────── + ──► out
/// ```
///
/// ## Comb Filter (feedback + one-pole lowpass in feedback loop)
///
/// Each of the 8 parallel comb filters uses a slightly different delay
/// length to build up a dense echo density. High-frequency content is
/// attenuated in each feedback path by a one-pole lowpass filter,
/// mimicking the way real rooms absorb high frequencies:
///
/// ```text
/// output     = delay[pos]
/// lp_store   = output × (1 − damp) + lp_store × damp
/// delay[pos] = input × FIXED_GAIN + lp_store × feedback
/// pos        = (pos + 1) mod length
/// ```
///
/// ## Allpass Filter (Schroeder, fixed coefficient 0.5)
///
/// ```text
/// bufout     = delay[pos]
/// delay[pos] = input + bufout × 0.5
/// output     = bufout − input
/// pos        = (pos + 1) mod length
/// ```
///
/// ## Parameters
///
/// | Node input  | Range | Typical | Notes                                               |
/// |-------------|-------|---------|-----------------------------------------------------|
/// | `room_size` | 0–1   | 0.84    | Feedback coefficient; values near 1 = long tail.   |
/// | `damping`   | 0–1   | 0.5     | High-frequency damping; 0 = bright, 1 = very dark. |
/// | `wet`       | 0–1   | 0.3     | Mix level of the reverb (processed) signal.        |
/// | `dry`       | 0–1   | 0.7     | Mix level of the direct (unprocessed) signal.      |
///
/// ## Modulation Stability
///
/// `room_size` and `damping` are passed through a one-pole smoothing
/// filter (coefficient `0.001`, ≈22 ms time constant at 44 100 Hz) before
/// entering the delay lines. This removes the step discontinuities that
/// would otherwise cause audible clicks or instability when these
/// parameters are modulated — for example by an [`ExternalFloatNode`] or a
/// [`SplineFloatNode`].
///
/// `wet` and `dry` affect only the final output mix and not any feedback
/// path, so they can be changed freely without risk of instability.
///
/// ## Delay Lines and Sample Rate
///
/// The internal delay lines are allocated lazily on the first call to
/// [`Source::batch_poll`]. Their lengths are the original Freeverb values
/// (measured at 44 100 Hz) scaled proportionally to whatever
/// `AudioContext::sample_rate` is in use, so the reverb character is
/// preserved at any sample rate.
///
/// ## References
///
/// - Jezar at Dreampoint: original C++ Freeverb implementation (2000).
/// - Julius O. Smith III, *Physical Audio Signal Processing*, "Freeverb"
///   chapter: <https://ccrma.stanford.edu/~jos/pasp/Freeverb.html>
pub struct FreeverbNode {
    id: usize,
    sample_source_id: usize,
    room_size_source_id: usize,
    damping_source_id: usize,
    wet_source_id: usize,
    dry_source_id: usize,
    dependency_ids: Vec<usize>,

    // Delay lines — None until the first batch_poll(), when sample_rate is known.
    combs: Option<Vec<CombFilter>>,
    allpasses: Option<Vec<AllpassFilter>>,

    // One-pole smoothed copies of room_size and damping.
    smoothed_room_size: f32,
    smoothed_damping: f32,
}

impl FreeverbNode {
    pub fn new(
        id: usize,
        sample_source_id: usize,
        room_size_source_id: usize,
        damping_source_id: usize,
        wet_source_id: usize,
        dry_source_id: usize,
    ) -> Self {
        FreeverbNode {
            id,
            sample_source_id,
            room_size_source_id,
            damping_source_id,
            wet_source_id,
            dry_source_id,
            dependency_ids: vec![
                sample_source_id,
                room_size_source_id,
                damping_source_id,
                wet_source_id,
                dry_source_id,
            ],
            combs: None,
            allpasses: None,
            smoothed_room_size: 0.5,
            smoothed_damping: 0.5,
        }
    }

    fn init_filters(&mut self, sample_rate: f32) {
        let scale = sample_rate / 44100.0;
        self.combs = Some(
            COMB_TUNINGS
                .iter()
                .map(|&t| CombFilter::new(((t as f32) * scale).round() as usize))
                .collect(),
        );
        self.allpasses = Some(
            ALLPASS_TUNINGS
                .iter()
                .map(|&t| AllpassFilter::new(((t as f32) * scale).round() as usize))
                .collect(),
        );
    }

    fn poll(
        &mut self,
        sample: Option<f32>,
        room_size: Option<f32>,
        damping: Option<f32>,
        wet: Option<f32>,
        dry: Option<f32>,
    ) -> Option<f32> {
        sample.zip(room_size).zip(damping).zip(wet).zip(dry).map(
            |((((sample, room_size), damping), wet), dry)| {
                // Smooth room_size and damping to suppress clicks under modulation.
                self.smoothed_room_size += SMOOTH_COEFF * (room_size - self.smoothed_room_size);
                self.smoothed_damping += SMOOTH_COEFF * (damping - self.smoothed_damping);

                let input = sample * FIXED_GAIN;

                // 8 parallel feedback comb filters.
                let comb_sum: f32 = self
                    .combs
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .map(|c| c.process(input, self.smoothed_room_size, self.smoothed_damping))
                    .sum();

                // 4 series allpass filters.
                let reverb_out = self
                    .allpasses
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .fold(comb_sum, |s, ap| ap.process(s));

                reverb_out * wet + sample * dry
            },
        )
    }
}

impl Source for FreeverbNode {
    fn batch_poll(
        &mut self,
        num_samples: usize,
        audio_context: &AudioContext,
        id_to_output: &NodeOutput,
        output: &mut [Option<f32>],
    ) {
        if self.combs.is_none() {
            self.init_filters(audio_context.sample_rate);
        }

        for idx in 0..num_samples {
            output[idx] = self.poll(
                id_to_output[self.sample_source_id][idx],
                id_to_output[self.room_size_source_id][idx],
                id_to_output[self.damping_source_id][idx],
                id_to_output[self.wet_source_id][idx],
                id_to_output[self.dry_source_id][idx],
            );
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn dependency_ids(&self) -> &Vec<usize> {
        &self.dependency_ids
    }
}
