#![allow(clippy::all)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]

/* ------------------------------------------------------------
name: "test"
Code generated with Faust 2.70.3 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */

use strum::FromRepr;

type F32 = f32;

fn sq(value: F32) -> F32 {
    return value * value;
}

#[repr(u8)]
#[derive(Debug, FromRepr)]
pub enum ParamIndex {
    DelayMs = 0,
    LowFreqCrossover = 1,
    T60Low = 2,
    T60Mid = 3,
    HighFreqDamping = 4,
    Eq1Freq = 5,
    Eq1Level = 6,
    Eq2Freq = 7,
    Eq2Level = 8,
    DryWetMix = 9,
    Level = 10,
}

#[derive(Clone)]
pub struct ZitaRev1 {
    fVslider0: F32,
    fVslider1: F32,
    fSampleRate: i32,
    fConst1: F32,
    fVslider2: F32,
    fVslider3: F32,
    fVslider4: F32,
    fConst3: F32,
    fVslider5: F32,
    fVslider6: F32,
    fConst4: F32,
    fRec13: [F32; 2],
    fVslider7: F32,
    fRec12: [F32; 2],
    IOTA0: i32,
    fVec0: [F32; 16384],
    iConst6: i32,
    fVec1: [F32; 16384],
    fVslider8: F32,
    fConst7: F32,
    fVec2: [F32; 4096],
    iConst8: i32,
    fRec10: [F32; 2],
    fConst10: F32,
    fRec17: [F32; 2],
    fRec16: [F32; 2],
    fVec3: [F32; 16384],
    iConst12: i32,
    fVec4: [F32; 2048],
    iConst13: i32,
    fRec14: [F32; 2],
    fConst15: F32,
    fRec21: [F32; 2],
    fRec20: [F32; 2],
    fVec5: [F32; 16384],
    iConst17: i32,
    fVec6: [F32; 4096],
    iConst18: i32,
    fRec18: [F32; 2],
    fConst20: F32,
    fRec25: [F32; 2],
    fRec24: [F32; 2],
    fVec7: [F32; 16384],
    iConst22: i32,
    fVec8: [F32; 2048],
    iConst23: i32,
    fRec22: [F32; 2],
    fConst25: F32,
    fRec29: [F32; 2],
    fRec28: [F32; 2],
    fVec9: [F32; 32768],
    iConst27: i32,
    fVec10: [F32; 16384],
    fVec11: [F32; 4096],
    iConst28: i32,
    fRec26: [F32; 2],
    fConst30: F32,
    fRec33: [F32; 2],
    fRec32: [F32; 2],
    fVec12: [F32; 16384],
    iConst32: i32,
    fVec13: [F32; 4096],
    iConst33: i32,
    fRec30: [F32; 2],
    fConst35: F32,
    fRec37: [F32; 2],
    fRec36: [F32; 2],
    fVec14: [F32; 32768],
    iConst37: i32,
    fVec15: [F32; 4096],
    iConst38: i32,
    fRec34: [F32; 2],
    fConst40: F32,
    fRec41: [F32; 2],
    fRec40: [F32; 2],
    fVec16: [F32; 32768],
    iConst42: i32,
    fVec17: [F32; 2048],
    iConst43: i32,
    fRec38: [F32; 2],
    fRec2: [F32; 3],
    fRec3: [F32; 3],
    fRec4: [F32; 3],
    fRec5: [F32; 3],
    fRec6: [F32; 3],
    fRec7: [F32; 3],
    fRec8: [F32; 3],
    fRec9: [F32; 3],
    fRec1: [F32; 3],
    fRec0: [F32; 3],
    fConst44: F32,
    fConst45: F32,
    fVslider9: F32,
    fRec42: [F32; 2],
    fVslider10: F32,
    fRec43: [F32; 2],
    fRec45: [F32; 3],
    fRec44: [F32; 3],
}

impl ZitaRev1 {
    pub fn new() -> ZitaRev1 {
        ZitaRev1 {
            fVslider0: 0.0,
            fVslider1: 0.0,
            fSampleRate: 0,
            fConst1: 0.0,
            fVslider2: 0.0,
            fVslider3: 0.0,
            fVslider4: 0.0,
            fConst3: 0.0,
            fVslider5: 0.0,
            fVslider6: 0.0,
            fConst4: 0.0,
            fRec13: [0.0; 2],
            fVslider7: 0.0,
            fRec12: [0.0; 2],
            IOTA0: 0,
            fVec0: [0.0; 16384],
            iConst6: 0,
            fVec1: [0.0; 16384],
            fVslider8: 0.0,
            fConst7: 0.0,
            fVec2: [0.0; 4096],
            iConst8: 0,
            fRec10: [0.0; 2],
            fConst10: 0.0,
            fRec17: [0.0; 2],
            fRec16: [0.0; 2],
            fVec3: [0.0; 16384],
            iConst12: 0,
            fVec4: [0.0; 2048],
            iConst13: 0,
            fRec14: [0.0; 2],
            fConst15: 0.0,
            fRec21: [0.0; 2],
            fRec20: [0.0; 2],
            fVec5: [0.0; 16384],
            iConst17: 0,
            fVec6: [0.0; 4096],
            iConst18: 0,
            fRec18: [0.0; 2],
            fConst20: 0.0,
            fRec25: [0.0; 2],
            fRec24: [0.0; 2],
            fVec7: [0.0; 16384],
            iConst22: 0,
            fVec8: [0.0; 2048],
            iConst23: 0,
            fRec22: [0.0; 2],
            fConst25: 0.0,
            fRec29: [0.0; 2],
            fRec28: [0.0; 2],
            fVec9: [0.0; 32768],
            iConst27: 0,
            fVec10: [0.0; 16384],
            fVec11: [0.0; 4096],
            iConst28: 0,
            fRec26: [0.0; 2],
            fConst30: 0.0,
            fRec33: [0.0; 2],
            fRec32: [0.0; 2],
            fVec12: [0.0; 16384],
            iConst32: 0,
            fVec13: [0.0; 4096],
            iConst33: 0,
            fRec30: [0.0; 2],
            fConst35: 0.0,
            fRec37: [0.0; 2],
            fRec36: [0.0; 2],
            fVec14: [0.0; 32768],
            iConst37: 0,
            fVec15: [0.0; 4096],
            iConst38: 0,
            fRec34: [0.0; 2],
            fConst40: 0.0,
            fRec41: [0.0; 2],
            fRec40: [0.0; 2],
            fVec16: [0.0; 32768],
            iConst42: 0,
            fVec17: [0.0; 2048],
            iConst43: 0,
            fRec38: [0.0; 2],
            fRec2: [0.0; 3],
            fRec3: [0.0; 3],
            fRec4: [0.0; 3],
            fRec5: [0.0; 3],
            fRec6: [0.0; 3],
            fRec7: [0.0; 3],
            fRec8: [0.0; 3],
            fRec9: [0.0; 3],
            fRec1: [0.0; 3],
            fRec0: [0.0; 3],
            fConst44: 0.0,
            fConst45: 0.0,
            fVslider9: 0.0,
            fRec42: [0.0; 2],
            fVslider10: 0.0,
            fRec43: [0.0; 2],
            fRec45: [0.0; 3],
            fRec44: [0.0; 3],
        }
    }

    // keep copyrights around
    // fn metadata(&self, m: &mut dyn Meta) {
    //     m.declare("basics.lib/name", r"Faust Basic Element Library");
    //     m.declare(
    //         "basics.lib/tabulateNd",
    //         r"Copyright (C) 2023 Bart Brouns <bart@magnetophon.nl>",
    //     );
    //     m.declare("basics.lib/version", r"1.12.0");
    //     m.declare(
    //         "compile_options",
    //         r"-lang rust -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0",
    //     );
    //     m.declare("delays.lib/name", r"Faust Delay Library");
    //     m.declare("delays.lib/version", r"1.1.0");
    //     m.declare("demos.lib/name", r"Faust Demos Library");
    //     m.declare("demos.lib/version", r"1.1.1");
    //     m.declare("demos.lib/zita_rev1:author", r"Julius O. Smith III");
    //     m.declare("demos.lib/zita_rev1:licence", r"MIT");
    //     m.declare("filename", r"test");
    //     m.declare("filters.lib/allpass_comb:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/allpass_comb:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/allpass_comb:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/fir:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/fir:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/fir:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/iir:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/iir:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/iir:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/lowpass0_highpass1", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/lowpass0_highpass1:author", r"Julius O. Smith III");
    //     m.declare("filters.lib/lowpass:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/lowpass:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/lowpass:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/name", r"Faust Filters Library");
    //     m.declare("filters.lib/peak_eq_rm:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/peak_eq_rm:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/peak_eq_rm:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/tf1:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/tf1:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/tf1:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/tf1s:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/tf1s:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/tf1s:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/tf2:author", r"Julius O. Smith III");
    //     m.declare(
    //         "filters.lib/tf2:copyright",
    //         r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>",
    //     );
    //     m.declare("filters.lib/tf2:license", r"MIT-style STK-4.3 license");
    //     m.declare("filters.lib/version", r"1.3.0");
    //     m.declare("maths.lib/author", r"GRAME");
    //     m.declare("maths.lib/copyright", r"GRAME");
    //     m.declare("maths.lib/license", r"LGPL with exception");
    //     m.declare("maths.lib/name", r"Faust Math Library");
    //     m.declare("maths.lib/version", r"2.7.0");
    //     m.declare("name", r"test");
    //     m.declare("platform.lib/name", r"Generic Platform Library");
    //     m.declare("platform.lib/version", r"1.3.0");
    //     m.declare("reverbs.lib/name", r"Faust Reverb Library");
    //     m.declare("reverbs.lib/version", r"1.2.1");
    //     m.declare("routes.lib/hadamard:author", r"Remy Muller, revised by Romain Michon");
    //     m.declare("routes.lib/name", r"Faust Signal Routing Library");
    //     m.declare("routes.lib/version", r"1.2.0");
    //     m.declare("signals.lib/name", r"Faust Signal Routing Library");
    //     m.declare("signals.lib/version", r"1.5.0");
    // }

    pub fn get_sample_rate(&self) -> i32 {
        return self.fSampleRate;
    }

    pub fn get_num_inputs(&self) -> i32 {
        return 2;
    }

    pub fn get_num_outputs(&self) -> i32 {
        return 2;
    }

    pub fn instance_reset_params(&mut self) {
        self.fVslider0 = 0.0;
        self.fVslider1 = 1.5e+03;
        self.fVslider2 = 0.0;
        self.fVslider3 = 315.0;
        self.fVslider4 = 2.0;
        self.fVslider5 = 6e+03;
        self.fVslider6 = 2e+02;
        self.fVslider7 = 3.0;
        self.fVslider8 = 6e+01;
        self.fVslider9 = 0.0;
        self.fVslider10 = -2e+01;
    }

    pub fn instance_clear(&mut self) {
        for l0 in 0..2 {
            self.fRec13[l0 as usize] = 0.0;
        }
        for l1 in 0..2 {
            self.fRec12[l1 as usize] = 0.0;
        }
        self.IOTA0 = 0;
        for l2 in 0..16384 {
            self.fVec0[l2 as usize] = 0.0;
        }
        for l3 in 0..16384 {
            self.fVec1[l3 as usize] = 0.0;
        }
        for l4 in 0..4096 {
            self.fVec2[l4 as usize] = 0.0;
        }
        for l5 in 0..2 {
            self.fRec10[l5 as usize] = 0.0;
        }
        for l6 in 0..2 {
            self.fRec17[l6 as usize] = 0.0;
        }
        for l7 in 0..2 {
            self.fRec16[l7 as usize] = 0.0;
        }
        for l8 in 0..16384 {
            self.fVec3[l8 as usize] = 0.0;
        }
        for l9 in 0..2048 {
            self.fVec4[l9 as usize] = 0.0;
        }
        for l10 in 0..2 {
            self.fRec14[l10 as usize] = 0.0;
        }
        for l11 in 0..2 {
            self.fRec21[l11 as usize] = 0.0;
        }
        for l12 in 0..2 {
            self.fRec20[l12 as usize] = 0.0;
        }
        for l13 in 0..16384 {
            self.fVec5[l13 as usize] = 0.0;
        }
        for l14 in 0..4096 {
            self.fVec6[l14 as usize] = 0.0;
        }
        for l15 in 0..2 {
            self.fRec18[l15 as usize] = 0.0;
        }
        for l16 in 0..2 {
            self.fRec25[l16 as usize] = 0.0;
        }
        for l17 in 0..2 {
            self.fRec24[l17 as usize] = 0.0;
        }
        for l18 in 0..16384 {
            self.fVec7[l18 as usize] = 0.0;
        }
        for l19 in 0..2048 {
            self.fVec8[l19 as usize] = 0.0;
        }
        for l20 in 0..2 {
            self.fRec22[l20 as usize] = 0.0;
        }
        for l21 in 0..2 {
            self.fRec29[l21 as usize] = 0.0;
        }
        for l22 in 0..2 {
            self.fRec28[l22 as usize] = 0.0;
        }
        for l23 in 0..32768 {
            self.fVec9[l23 as usize] = 0.0;
        }
        for l24 in 0..16384 {
            self.fVec10[l24 as usize] = 0.0;
        }
        for l25 in 0..4096 {
            self.fVec11[l25 as usize] = 0.0;
        }
        for l26 in 0..2 {
            self.fRec26[l26 as usize] = 0.0;
        }
        for l27 in 0..2 {
            self.fRec33[l27 as usize] = 0.0;
        }
        for l28 in 0..2 {
            self.fRec32[l28 as usize] = 0.0;
        }
        for l29 in 0..16384 {
            self.fVec12[l29 as usize] = 0.0;
        }
        for l30 in 0..4096 {
            self.fVec13[l30 as usize] = 0.0;
        }
        for l31 in 0..2 {
            self.fRec30[l31 as usize] = 0.0;
        }
        for l32 in 0..2 {
            self.fRec37[l32 as usize] = 0.0;
        }
        for l33 in 0..2 {
            self.fRec36[l33 as usize] = 0.0;
        }
        for l34 in 0..32768 {
            self.fVec14[l34 as usize] = 0.0;
        }
        for l35 in 0..4096 {
            self.fVec15[l35 as usize] = 0.0;
        }
        for l36 in 0..2 {
            self.fRec34[l36 as usize] = 0.0;
        }
        for l37 in 0..2 {
            self.fRec41[l37 as usize] = 0.0;
        }
        for l38 in 0..2 {
            self.fRec40[l38 as usize] = 0.0;
        }
        for l39 in 0..32768 {
            self.fVec16[l39 as usize] = 0.0;
        }
        for l40 in 0..2048 {
            self.fVec17[l40 as usize] = 0.0;
        }
        for l41 in 0..2 {
            self.fRec38[l41 as usize] = 0.0;
        }
        for l42 in 0..3 {
            self.fRec2[l42 as usize] = 0.0;
        }
        for l43 in 0..3 {
            self.fRec3[l43 as usize] = 0.0;
        }
        for l44 in 0..3 {
            self.fRec4[l44 as usize] = 0.0;
        }
        for l45 in 0..3 {
            self.fRec5[l45 as usize] = 0.0;
        }
        for l46 in 0..3 {
            self.fRec6[l46 as usize] = 0.0;
        }
        for l47 in 0..3 {
            self.fRec7[l47 as usize] = 0.0;
        }
        for l48 in 0..3 {
            self.fRec8[l48 as usize] = 0.0;
        }
        for l49 in 0..3 {
            self.fRec9[l49 as usize] = 0.0;
        }
        for l50 in 0..3 {
            self.fRec1[l50 as usize] = 0.0;
        }
        for l51 in 0..3 {
            self.fRec0[l51 as usize] = 0.0;
        }
        for l52 in 0..2 {
            self.fRec42[l52 as usize] = 0.0;
        }
        for l53 in 0..2 {
            self.fRec43[l53 as usize] = 0.0;
        }
        for l54 in 0..3 {
            self.fRec45[l54 as usize] = 0.0;
        }
        for l55 in 0..3 {
            self.fRec44[l55 as usize] = 0.0;
        }
    }

    pub fn instance_constants(&mut self, sample_rate: i32) {
        self.fSampleRate = sample_rate;
        let mut fConst0: F32 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
        self.fConst1 = 6.2831855 / fConst0;
        let mut fConst2: F32 = F32::floor(0.174713 * fConst0 + 0.5);
        self.fConst3 = 6.9077554 * (fConst2 / fConst0);
        self.fConst4 = 3.1415927 / fConst0;
        let mut fConst5: F32 = F32::floor(0.022904 * fConst0 + 0.5);
        self.iConst6 = (F32::min(8192.0, F32::max(0.0, fConst2 - fConst5))) as i32;
        self.fConst7 = 0.001 * fConst0;
        self.iConst8 = (F32::min(2048.0, F32::max(0.0, fConst5 + -1.0))) as i32;
        let mut fConst9: F32 = F32::floor(0.153129 * fConst0 + 0.5);
        self.fConst10 = 6.9077554 * (fConst9 / fConst0);
        let mut fConst11: F32 = F32::floor(0.020346 * fConst0 + 0.5);
        self.iConst12 = (F32::min(8192.0, F32::max(0.0, fConst9 - fConst11))) as i32;
        self.iConst13 = (F32::min(1024.0, F32::max(0.0, fConst11 + -1.0))) as i32;
        let mut fConst14: F32 = F32::floor(0.127837 * fConst0 + 0.5);
        self.fConst15 = 6.9077554 * (fConst14 / fConst0);
        let mut fConst16: F32 = F32::floor(0.031604 * fConst0 + 0.5);
        self.iConst17 = (F32::min(8192.0, F32::max(0.0, fConst14 - fConst16))) as i32;
        self.iConst18 = (F32::min(2048.0, F32::max(0.0, fConst16 + -1.0))) as i32;
        let mut fConst19: F32 = F32::floor(0.125 * fConst0 + 0.5);
        self.fConst20 = 6.9077554 * (fConst19 / fConst0);
        let mut fConst21: F32 = F32::floor(0.013458 * fConst0 + 0.5);
        self.iConst22 = (F32::min(8192.0, F32::max(0.0, fConst19 - fConst21))) as i32;
        self.iConst23 = (F32::min(1024.0, F32::max(0.0, fConst21 + -1.0))) as i32;
        let mut fConst24: F32 = F32::floor(0.210389 * fConst0 + 0.5);
        self.fConst25 = 6.9077554 * (fConst24 / fConst0);
        let mut fConst26: F32 = F32::floor(0.024421 * fConst0 + 0.5);
        self.iConst27 = (F32::min(16384.0, F32::max(0.0, fConst24 - fConst26))) as i32;
        self.iConst28 = (F32::min(2048.0, F32::max(0.0, fConst26 + -1.0))) as i32;
        let mut fConst29: F32 = F32::floor(0.192303 * fConst0 + 0.5);
        self.fConst30 = 6.9077554 * (fConst29 / fConst0);
        let mut fConst31: F32 = F32::floor(0.029291 * fConst0 + 0.5);
        self.iConst32 = (F32::min(8192.0, F32::max(0.0, fConst29 - fConst31))) as i32;
        self.iConst33 = (F32::min(2048.0, F32::max(0.0, fConst31 + -1.0))) as i32;
        let mut fConst34: F32 = F32::floor(0.256891 * fConst0 + 0.5);
        self.fConst35 = 6.9077554 * (fConst34 / fConst0);
        let mut fConst36: F32 = F32::floor(0.027333 * fConst0 + 0.5);
        self.iConst37 = (F32::min(16384.0, F32::max(0.0, fConst34 - fConst36))) as i32;
        self.iConst38 = (F32::min(2048.0, F32::max(0.0, fConst36 + -1.0))) as i32;
        let mut fConst39: F32 = F32::floor(0.219991 * fConst0 + 0.5);
        self.fConst40 = 6.9077554 * (fConst39 / fConst0);
        let mut fConst41: F32 = F32::floor(0.019123 * fConst0 + 0.5);
        self.iConst42 = (F32::min(16384.0, F32::max(0.0, fConst39 - fConst41))) as i32;
        self.iConst43 = (F32::min(1024.0, F32::max(0.0, fConst41 + -1.0))) as i32;
        self.fConst44 = 44.1 / fConst0;
        self.fConst45 = 1.0 - self.fConst44;
    }

    pub fn instance_init(&mut self, sample_rate: i32) {
        self.instance_constants(sample_rate);
        self.instance_reset_params();
        self.instance_clear();
    }

    pub fn init(&mut self, sample_rate: i32) {
        self.instance_init(sample_rate);
    }

    // fn build_user_interface_static(ui_interface: &mut dyn UI<Self::T>) {
    //     ui_interface.declare(None, "0", "");
    //     ui_interface.declare(None, "tooltip", "~ ZITA REV1 FEEDBACK DELAY NETWORK (FDN) & SCHROEDER     ALLPASS-COMB REVERBERATOR (8x8). See Faust's reverbs.lib for documentation and     references");
    //     ui_interface.open_horizontal_box("Zita_Rev1");
    //     ui_interface.declare(None, "1", "");
    //     ui_interface.open_horizontal_box("Input");
    //     ui_interface.declare(Some(ParamIndex(0)), "1", "");
    //     ui_interface.declare(Some(ParamIndex(0)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(0)),
    //         "tooltip",
    //         "Delay in ms         before reverberation begins",
    //     );
    //     ui_interface.declare(Some(ParamIndex(0)), "unit", "ms");
    //     ui_interface.add_vertical_slider("In Delay", ParamIndex(0), 6e+01, 2e+01, 1e+02, 1.0);
    //     ui_interface.close_box();
    //     ui_interface.declare(None, "2", "");
    //     ui_interface.open_horizontal_box("Decay Times in Bands (see tooltips)");
    //     ui_interface.declare(Some(ParamIndex(1)), "1", "");
    //     ui_interface.declare(Some(ParamIndex(1)), "scale", "log");
    //     ui_interface.declare(Some(ParamIndex(1)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(1)),
    //         "tooltip",
    //         "Crossover frequency (Hz) separating low and middle frequencies",
    //     );
    //     ui_interface.declare(Some(ParamIndex(1)), "unit", "Hz");
    //     ui_interface.add_vertical_slider("LF X", ParamIndex(1), 2e+02, 5e+01, 1e+03, 1.0);
    //     ui_interface.declare(Some(ParamIndex(2)), "2", "");
    //     ui_interface.declare(Some(ParamIndex(2)), "scale", "log");
    //     ui_interface.declare(Some(ParamIndex(2)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(2)),
    //         "tooltip",
    //         "T60 = time (in seconds) to decay 60dB in low-frequency band",
    //     );
    //     ui_interface.declare(Some(ParamIndex(2)), "unit", "s");
    //     ui_interface.add_vertical_slider("Low RT60", ParamIndex(2), 3.0, 1.0, 8.0, 0.1);
    //     ui_interface.declare(Some(ParamIndex(3)), "3", "");
    //     ui_interface.declare(Some(ParamIndex(3)), "scale", "log");
    //     ui_interface.declare(Some(ParamIndex(3)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(3)),
    //         "tooltip",
    //         "T60 = time (in seconds) to decay 60dB in middle band",
    //     );
    //     ui_interface.declare(Some(ParamIndex(3)), "unit", "s");
    //     ui_interface.add_vertical_slider("Mid RT60", ParamIndex(3), 2.0, 1.0, 8.0, 0.1);
    //     ui_interface.declare(Some(ParamIndex(4)), "4", "");
    //     ui_interface.declare(Some(ParamIndex(4)), "scale", "log");
    //     ui_interface.declare(Some(ParamIndex(4)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(4)),
    //         "tooltip",
    //         "Frequency (Hz) at which the high-frequency T60 is half the middle-band's T60",
    //     );
    //     ui_interface.declare(Some(ParamIndex(4)), "unit", "Hz");
    //     ui_interface.add_vertical_slider("HF Damping", ParamIndex(4), 6e+03, 1.5e+03, 2.352e+04, 1.0);
    //     ui_interface.close_box();
    //     ui_interface.declare(None, "3", "");
    //     ui_interface.open_horizontal_box("RM Peaking Equalizer 1");
    //     ui_interface.declare(Some(ParamIndex(5)), "1", "");
    //     ui_interface.declare(Some(ParamIndex(5)), "scale", "log");
    //     ui_interface.declare(Some(ParamIndex(5)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(5)),
    //         "tooltip",
    //         "Center-frequency of second-order Regalia-Mitra peaking equalizer section 1",
    //     );
    //     ui_interface.declare(Some(ParamIndex(5)), "unit", "Hz");
    //     ui_interface.add_vertical_slider("Eq1 Freq", ParamIndex(5), 315.0, 4e+01, 2.5e+03, 1.0);
    //     ui_interface.declare(Some(ParamIndex(6)), "2", "");
    //     ui_interface.declare(Some(ParamIndex(6)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(6)),
    //         "tooltip",
    //         "Peak level         in dB of second-order Regalia-Mitra peaking equalizer section 1",
    //     );
    //     ui_interface.declare(Some(ParamIndex(6)), "unit", "dB");
    //     ui_interface.add_vertical_slider("Eq1 Level", ParamIndex(6), 0.0, -15.0, 15.0, 0.1);
    //     ui_interface.close_box();
    //     ui_interface.declare(None, "4", "");
    //     ui_interface.open_horizontal_box("RM Peaking Equalizer 2");
    //     ui_interface.declare(Some(ParamIndex(7)), "1", "");
    //     ui_interface.declare(Some(ParamIndex(7)), "scale", "log");
    //     ui_interface.declare(Some(ParamIndex(7)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(7)),
    //         "tooltip",
    //         "Center-frequency of second-order Regalia-Mitra peaking equalizer section 2",
    //     );
    //     ui_interface.declare(Some(ParamIndex(7)), "unit", "Hz");
    //     ui_interface.add_vertical_slider("Eq2 Freq", ParamIndex(7), 1.5e+03, 1.6e+02, 1e+04, 1.0);
    //     ui_interface.declare(Some(ParamIndex(8)), "2", "");
    //     ui_interface.declare(Some(ParamIndex(8)), "style", "knob");
    //     ui_interface.declare(
    //         Some(ParamIndex(8)),
    //         "tooltip",
    //         "Peak level         in dB of second-order Regalia-Mitra peaking equalizer section 2",
    //     );
    //     ui_interface.declare(Some(ParamIndex(8)), "unit", "dB");
    //     ui_interface.add_vertical_slider("Eq2 Level", ParamIndex(8), 0.0, -15.0, 15.0, 0.1);
    //     ui_interface.close_box();
    //     ui_interface.declare(None, "5", "");
    //     ui_interface.open_horizontal_box("Output");
    //     ui_interface.declare(Some(ParamIndex(9)), "1", "");
    //     ui_interface.declare(Some(ParamIndex(9)), "style", "knob");
    //     ui_interface.declare(Some(ParamIndex(9)), "tooltip", "-1 = dry, 1 = wet");
    //     ui_interface.add_vertical_slider("Dry/Wet Mix", ParamIndex(9), 0.0, -1.0, 1.0, 0.01);
    //     ui_interface.declare(Some(ParamIndex(10)), "2", "");
    //     ui_interface.declare(Some(ParamIndex(10)), "style", "knob");
    //     ui_interface.declare(Some(ParamIndex(10)), "tooltip", "Output scale         factor");
    //     ui_interface.declare(Some(ParamIndex(10)), "unit", "dB");
    //     ui_interface.add_vertical_slider("Level", ParamIndex(10), -2e+01, -7e+01, 4e+01, 0.1);
    //     ui_interface.close_box();
    //     ui_interface.close_box();
    // }

    pub fn get_param(&self, param: ParamIndex) -> Option<f32> {
        match param as u8 {
            8 => Some(self.fVslider0),
            7 => Some(self.fVslider1),
            10 => Some(self.fVslider10),
            6 => Some(self.fVslider2),
            5 => Some(self.fVslider3),
            3 => Some(self.fVslider4),
            4 => Some(self.fVslider5),
            1 => Some(self.fVslider6),
            2 => Some(self.fVslider7),
            0 => Some(self.fVslider8),
            9 => Some(self.fVslider9),
            _ => None,
        }
    }

    pub fn set_param(&mut self, param: ParamIndex, value: f32) {
        match param as u8 {
            8 => self.fVslider0 = value,
            7 => self.fVslider1 = value,
            10 => self.fVslider10 = value,
            6 => self.fVslider2 = value,
            5 => self.fVslider3 = value,
            3 => self.fVslider4 = value,
            4 => self.fVslider5 = value,
            1 => self.fVslider6 = value,
            2 => self.fVslider7 = value,
            0 => self.fVslider8 = value,
            9 => self.fVslider9 = value,
            _ => {}
        }
    }

    pub fn compute(
        &mut self,
        count: i32,
        inputs0: &[f32],
        inputs1: &[f32],
        outputs0: &mut [f32],
        outputs1: &mut [f32],
    ) {
        let mut fSlow0: F32 = F32::powf(1e+01, 0.05 * self.fVslider0);
        let mut fSlow1: F32 = self.fVslider1;
        let mut fSlow2: F32 = self.fConst1 * (fSlow1 / F32::sqrt(F32::max(0.0, fSlow0)));
        let mut fSlow3: F32 = (1.0 - fSlow2) / (fSlow2 + 1.0);
        let mut fSlow4: F32 = F32::cos(self.fConst1 * fSlow1) * (fSlow3 + 1.0);
        let mut fSlow5: F32 = F32::powf(1e+01, 0.05 * self.fVslider2);
        let mut fSlow6: F32 = self.fVslider3;
        let mut fSlow7: F32 = self.fConst1 * (fSlow6 / F32::sqrt(F32::max(0.0, fSlow5)));
        let mut fSlow8: F32 = (1.0 - fSlow7) / (fSlow7 + 1.0);
        let mut fSlow9: F32 = F32::cos(self.fConst1 * fSlow6) * (fSlow8 + 1.0);
        let mut fSlow10: F32 = self.fVslider4;
        let mut fSlow11: F32 = F32::exp(-(self.fConst3 / fSlow10));
        let mut fSlow12: F32 = sq(fSlow11);
        let mut fSlow13: F32 = 1.0 - fSlow12;
        let mut fSlow14: F32 = F32::cos(self.fConst1 * self.fVslider5);
        let mut fSlow15: F32 = 1.0 - fSlow14 * fSlow12;
        let mut fSlow16: F32 = F32::sqrt(F32::max(0.0, sq(fSlow15) / sq(fSlow13) + -1.0));
        let mut fSlow17: F32 = fSlow15 / fSlow13;
        let mut fSlow18: F32 = fSlow17 - fSlow16;
        let mut fSlow19: F32 = 1.0 / F32::tan(self.fConst4 * self.fVslider6);
        let mut fSlow20: F32 = 1.0 - fSlow19;
        let mut fSlow21: F32 = 1.0 / (fSlow19 + 1.0);
        let mut fSlow22: F32 = self.fVslider7;
        let mut fSlow23: F32 = F32::exp(-(self.fConst3 / fSlow22)) / fSlow11 + -1.0;
        let mut fSlow24: F32 = fSlow11 * (fSlow16 + (1.0 - fSlow17));
        let mut iSlow25: i32 = (F32::min(8192.0, F32::max(0.0, self.fConst7 * self.fVslider8))) as i32;
        let mut fSlow26: F32 = F32::exp(-(self.fConst10 / fSlow10));
        let mut fSlow27: F32 = sq(fSlow26);
        let mut fSlow28: F32 = 1.0 - fSlow27;
        let mut fSlow29: F32 = 1.0 - fSlow27 * fSlow14;
        let mut fSlow30: F32 = F32::sqrt(F32::max(0.0, sq(fSlow29) / sq(fSlow28) + -1.0));
        let mut fSlow31: F32 = fSlow29 / fSlow28;
        let mut fSlow32: F32 = fSlow31 - fSlow30;
        let mut fSlow33: F32 = F32::exp(-(self.fConst10 / fSlow22)) / fSlow26 + -1.0;
        let mut fSlow34: F32 = fSlow26 * (fSlow30 + (1.0 - fSlow31));
        let mut fSlow35: F32 = F32::exp(-(self.fConst15 / fSlow10));
        let mut fSlow36: F32 = sq(fSlow35);
        let mut fSlow37: F32 = 1.0 - fSlow36;
        let mut fSlow38: F32 = 1.0 - fSlow14 * fSlow36;
        let mut fSlow39: F32 = F32::sqrt(F32::max(0.0, sq(fSlow38) / sq(fSlow37) + -1.0));
        let mut fSlow40: F32 = fSlow38 / fSlow37;
        let mut fSlow41: F32 = fSlow40 - fSlow39;
        let mut fSlow42: F32 = F32::exp(-(self.fConst15 / fSlow22)) / fSlow35 + -1.0;
        let mut fSlow43: F32 = fSlow35 * (fSlow39 + (1.0 - fSlow40));
        let mut fSlow44: F32 = F32::exp(-(self.fConst20 / fSlow10));
        let mut fSlow45: F32 = sq(fSlow44);
        let mut fSlow46: F32 = 1.0 - fSlow45;
        let mut fSlow47: F32 = 1.0 - fSlow14 * fSlow45;
        let mut fSlow48: F32 = F32::sqrt(F32::max(0.0, sq(fSlow47) / sq(fSlow46) + -1.0));
        let mut fSlow49: F32 = fSlow47 / fSlow46;
        let mut fSlow50: F32 = fSlow49 - fSlow48;
        let mut fSlow51: F32 = F32::exp(-(self.fConst20 / fSlow22)) / fSlow44 + -1.0;
        let mut fSlow52: F32 = fSlow44 * (fSlow48 + (1.0 - fSlow49));
        let mut fSlow53: F32 = F32::exp(-(self.fConst25 / fSlow10));
        let mut fSlow54: F32 = sq(fSlow53);
        let mut fSlow55: F32 = 1.0 - fSlow54;
        let mut fSlow56: F32 = 1.0 - fSlow14 * fSlow54;
        let mut fSlow57: F32 = F32::sqrt(F32::max(0.0, sq(fSlow56) / sq(fSlow55) + -1.0));
        let mut fSlow58: F32 = fSlow56 / fSlow55;
        let mut fSlow59: F32 = fSlow58 - fSlow57;
        let mut fSlow60: F32 = F32::exp(-(self.fConst25 / fSlow22)) / fSlow53 + -1.0;
        let mut fSlow61: F32 = fSlow53 * (fSlow57 + (1.0 - fSlow58));
        let mut fSlow62: F32 = F32::exp(-(self.fConst30 / fSlow10));
        let mut fSlow63: F32 = sq(fSlow62);
        let mut fSlow64: F32 = 1.0 - fSlow63;
        let mut fSlow65: F32 = 1.0 - fSlow14 * fSlow63;
        let mut fSlow66: F32 = F32::sqrt(F32::max(0.0, sq(fSlow65) / sq(fSlow64) + -1.0));
        let mut fSlow67: F32 = fSlow65 / fSlow64;
        let mut fSlow68: F32 = fSlow67 - fSlow66;
        let mut fSlow69: F32 = F32::exp(-(self.fConst30 / fSlow22)) / fSlow62 + -1.0;
        let mut fSlow70: F32 = fSlow62 * (fSlow66 + (1.0 - fSlow67));
        let mut fSlow71: F32 = F32::exp(-(self.fConst35 / fSlow10));
        let mut fSlow72: F32 = sq(fSlow71);
        let mut fSlow73: F32 = 1.0 - fSlow72;
        let mut fSlow74: F32 = 1.0 - fSlow14 * fSlow72;
        let mut fSlow75: F32 = F32::sqrt(F32::max(0.0, sq(fSlow74) / sq(fSlow73) + -1.0));
        let mut fSlow76: F32 = fSlow74 / fSlow73;
        let mut fSlow77: F32 = fSlow76 - fSlow75;
        let mut fSlow78: F32 = F32::exp(-(self.fConst35 / fSlow22)) / fSlow71 + -1.0;
        let mut fSlow79: F32 = fSlow71 * (fSlow75 + (1.0 - fSlow76));
        let mut fSlow80: F32 = F32::exp(-(self.fConst40 / fSlow10));
        let mut fSlow81: F32 = sq(fSlow80);
        let mut fSlow82: F32 = 1.0 - fSlow81;
        let mut fSlow83: F32 = 1.0 - fSlow14 * fSlow81;
        let mut fSlow84: F32 = F32::sqrt(F32::max(0.0, sq(fSlow83) / sq(fSlow82) + -1.0));
        let mut fSlow85: F32 = fSlow83 / fSlow82;
        let mut fSlow86: F32 = fSlow85 - fSlow84;
        let mut fSlow87: F32 = F32::exp(-(self.fConst40 / fSlow22)) / fSlow80 + -1.0;
        let mut fSlow88: F32 = fSlow80 * (fSlow84 + (1.0 - fSlow85));
        let mut fSlow89: F32 = self.fConst44 * self.fVslider9;
        let mut fSlow90: F32 = self.fConst44 * F32::powf(1e+01, 0.05 * self.fVslider10);
        let zipped_iterators = inputs0
            .iter()
            .zip(inputs1.iter())
            .zip(outputs0.iter_mut())
            .zip(outputs1.iter_mut());
        for (((input0, input1), output0), output1) in zipped_iterators {
            let mut fTemp0: F32 = fSlow4 * self.fRec0[1];
            let mut fTemp1: F32 = fSlow9 * self.fRec1[1];
            self.fRec13[0] = -(fSlow21 * (fSlow20 * self.fRec13[1] - (self.fRec6[1] + self.fRec6[2])));
            self.fRec12[0] = fSlow24 * (self.fRec6[1] + fSlow23 * self.fRec13[0]) + fSlow18 * self.fRec12[1];
            self.fVec0[(self.IOTA0 & 16383) as usize] = 0.35355338 * self.fRec12[0] + 1e-20;
            let mut fTemp2: F32 = *input0;
            self.fVec1[(self.IOTA0 & 16383) as usize] = fTemp2;
            let mut fTemp3: F32 = 0.3 * self.fVec1[((i32::wrapping_sub(self.IOTA0, iSlow25)) & 16383) as usize];
            let mut fTemp4: F32 = fTemp3 + self.fVec0[((i32::wrapping_sub(self.IOTA0, self.iConst6)) & 16383) as usize]
                - 0.6 * self.fRec10[1];
            self.fVec2[(self.IOTA0 & 4095) as usize] = fTemp4;
            self.fRec10[0] = self.fVec2[((i32::wrapping_sub(self.IOTA0, self.iConst8)) & 4095) as usize];
            let mut fRec11: F32 = 0.6 * fTemp4;
            self.fRec17[0] = -(fSlow21 * (fSlow20 * self.fRec17[1] - (self.fRec2[1] + self.fRec2[2])));
            self.fRec16[0] = fSlow34 * (self.fRec2[1] + fSlow33 * self.fRec17[0]) + fSlow32 * self.fRec16[1];
            self.fVec3[(self.IOTA0 & 16383) as usize] = 0.35355338 * self.fRec16[0] + 1e-20;
            let mut fTemp5: F32 = self.fVec3[((i32::wrapping_sub(self.IOTA0, self.iConst12)) & 16383) as usize]
                + fTemp3
                - 0.6 * self.fRec14[1];
            self.fVec4[(self.IOTA0 & 2047) as usize] = fTemp5;
            self.fRec14[0] = self.fVec4[((i32::wrapping_sub(self.IOTA0, self.iConst13)) & 2047) as usize];
            let mut fRec15: F32 = 0.6 * fTemp5;
            let mut fTemp6: F32 = fRec15 + fRec11;
            self.fRec21[0] = -(fSlow21 * (fSlow20 * self.fRec21[1] - (self.fRec4[1] + self.fRec4[2])));
            self.fRec20[0] = fSlow43 * (self.fRec4[1] + fSlow42 * self.fRec21[0]) + fSlow41 * self.fRec20[1];
            self.fVec5[(self.IOTA0 & 16383) as usize] = 0.35355338 * self.fRec20[0] + 1e-20;
            let mut fTemp7: F32 = self.fVec5[((i32::wrapping_sub(self.IOTA0, self.iConst17)) & 16383) as usize]
                - (fTemp3 + 0.6 * self.fRec18[1]);
            self.fVec6[(self.IOTA0 & 4095) as usize] = fTemp7;
            self.fRec18[0] = self.fVec6[((i32::wrapping_sub(self.IOTA0, self.iConst18)) & 4095) as usize];
            let mut fRec19: F32 = 0.6 * fTemp7;
            self.fRec25[0] = -(fSlow21 * (fSlow20 * self.fRec25[1] - (self.fRec8[1] + self.fRec8[2])));
            self.fRec24[0] = fSlow52 * (self.fRec8[1] + fSlow51 * self.fRec25[0]) + fSlow50 * self.fRec24[1];
            self.fVec7[(self.IOTA0 & 16383) as usize] = 0.35355338 * self.fRec24[0] + 1e-20;
            let mut fTemp8: F32 = self.fVec7[((i32::wrapping_sub(self.IOTA0, self.iConst22)) & 16383) as usize]
                - (fTemp3 + 0.6 * self.fRec22[1]);
            self.fVec8[(self.IOTA0 & 2047) as usize] = fTemp8;
            self.fRec22[0] = self.fVec8[((i32::wrapping_sub(self.IOTA0, self.iConst23)) & 2047) as usize];
            let mut fRec23: F32 = 0.6 * fTemp8;
            let mut fTemp9: F32 = fRec23 + fRec19 + fTemp6;
            self.fRec29[0] = -(fSlow21 * (fSlow20 * self.fRec29[1] - (self.fRec3[1] + self.fRec3[2])));
            self.fRec28[0] = fSlow61 * (self.fRec3[1] + fSlow60 * self.fRec29[0]) + fSlow59 * self.fRec28[1];
            self.fVec9[(self.IOTA0 & 32767) as usize] = 0.35355338 * self.fRec28[0] + 1e-20;
            let mut fTemp10: F32 = *input1;
            self.fVec10[(self.IOTA0 & 16383) as usize] = fTemp10;
            let mut fTemp11: F32 = 0.3 * self.fVec10[((i32::wrapping_sub(self.IOTA0, iSlow25)) & 16383) as usize];
            let mut fTemp12: F32 = fTemp11
                + 0.6 * self.fRec26[1]
                + self.fVec9[((i32::wrapping_sub(self.IOTA0, self.iConst27)) & 32767) as usize];
            self.fVec11[(self.IOTA0 & 4095) as usize] = fTemp12;
            self.fRec26[0] = self.fVec11[((i32::wrapping_sub(self.IOTA0, self.iConst28)) & 4095) as usize];
            let mut fRec27: F32 = -(0.6 * fTemp12);
            self.fRec33[0] = -(fSlow21 * (fSlow20 * self.fRec33[1] - (self.fRec7[1] + self.fRec7[2])));
            self.fRec32[0] = fSlow70 * (self.fRec7[1] + fSlow69 * self.fRec33[0]) + fSlow68 * self.fRec32[1];
            self.fVec12[(self.IOTA0 & 16383) as usize] = 0.35355338 * self.fRec32[0] + 1e-20;
            let mut fTemp13: F32 = self.fVec12[((i32::wrapping_sub(self.IOTA0, self.iConst32)) & 16383) as usize]
                + fTemp11
                + 0.6 * self.fRec30[1];
            self.fVec13[(self.IOTA0 & 4095) as usize] = fTemp13;
            self.fRec30[0] = self.fVec13[((i32::wrapping_sub(self.IOTA0, self.iConst33)) & 4095) as usize];
            let mut fRec31: F32 = -(0.6 * fTemp13);
            self.fRec37[0] = -(fSlow21 * (fSlow20 * self.fRec37[1] - (self.fRec5[1] + self.fRec5[2])));
            self.fRec36[0] = fSlow79 * (self.fRec5[1] + fSlow78 * self.fRec37[0]) + fSlow77 * self.fRec36[1];
            self.fVec14[(self.IOTA0 & 32767) as usize] = 0.35355338 * self.fRec36[0] + 1e-20;
            let mut fTemp14: F32 =
                0.6 * self.fRec34[1] + self.fVec14[((i32::wrapping_sub(self.IOTA0, self.iConst37)) & 32767) as usize];
            self.fVec15[(self.IOTA0 & 4095) as usize] = fTemp14 - fTemp11;
            self.fRec34[0] = self.fVec15[((i32::wrapping_sub(self.IOTA0, self.iConst38)) & 4095) as usize];
            let mut fRec35: F32 = 0.6 * (fTemp11 - fTemp14);
            self.fRec41[0] = -(fSlow21 * (fSlow20 * self.fRec41[1] - (self.fRec9[1] + self.fRec9[2])));
            self.fRec40[0] = fSlow88 * (self.fRec9[1] + fSlow87 * self.fRec41[0]) + fSlow86 * self.fRec40[1];
            self.fVec16[(self.IOTA0 & 32767) as usize] = 0.35355338 * self.fRec40[0] + 1e-20;
            let mut fTemp15: F32 =
                0.6 * self.fRec38[1] + self.fVec16[((i32::wrapping_sub(self.IOTA0, self.iConst42)) & 32767) as usize];
            self.fVec17[(self.IOTA0 & 2047) as usize] = fTemp15 - fTemp11;
            self.fRec38[0] = self.fVec17[((i32::wrapping_sub(self.IOTA0, self.iConst43)) & 2047) as usize];
            let mut fRec39: F32 = 0.6 * (fTemp11 - fTemp15);
            self.fRec2[0] = self.fRec38[1]
                + self.fRec34[1]
                + self.fRec30[1]
                + self.fRec26[1]
                + self.fRec22[1]
                + self.fRec18[1]
                + self.fRec10[1]
                + self.fRec14[1]
                + fRec39
                + fRec35
                + fRec31
                + fRec27
                + fTemp9;
            self.fRec3[0] = self.fRec22[1] + self.fRec18[1] + self.fRec10[1] + self.fRec14[1] + fTemp9
                - (self.fRec38[1]
                    + self.fRec34[1]
                    + self.fRec30[1]
                    + self.fRec26[1]
                    + fRec39
                    + fRec35
                    + fRec27
                    + fRec31);
            let mut fTemp16: F32 = fRec19 + fRec23;
            self.fRec4[0] =
                self.fRec30[1] + self.fRec26[1] + self.fRec10[1] + self.fRec14[1] + fRec31 + fRec27 + fTemp6
                    - (self.fRec38[1] + self.fRec34[1] + self.fRec22[1] + self.fRec18[1] + fRec39 + fRec35 + fTemp16);
            self.fRec5[0] =
                self.fRec38[1] + self.fRec34[1] + self.fRec10[1] + self.fRec14[1] + fRec39 + fRec35 + fTemp6
                    - (self.fRec30[1] + self.fRec26[1] + self.fRec22[1] + self.fRec18[1] + fRec31 + fRec27 + fTemp16);
            let mut fTemp17: F32 = fRec11 + fRec23;
            let mut fTemp18: F32 = fRec15 + fRec19;
            self.fRec6[0] =
                self.fRec34[1] + self.fRec26[1] + self.fRec18[1] + self.fRec14[1] + fRec35 + fRec27 + fTemp18
                    - (self.fRec38[1] + self.fRec30[1] + self.fRec22[1] + self.fRec10[1] + fRec39 + fRec31 + fTemp17);
            self.fRec7[0] =
                self.fRec38[1] + self.fRec30[1] + self.fRec18[1] + self.fRec14[1] + fRec39 + fRec31 + fTemp18
                    - (self.fRec34[1] + self.fRec26[1] + self.fRec22[1] + self.fRec10[1] + fRec35 + fRec27 + fTemp17);
            let mut fTemp19: F32 = fRec11 + fRec19;
            let mut fTemp20: F32 = fRec15 + fRec23;
            self.fRec8[0] =
                self.fRec38[1] + self.fRec26[1] + self.fRec22[1] + self.fRec14[1] + fRec39 + fRec27 + fTemp20
                    - (self.fRec34[1] + self.fRec30[1] + self.fRec18[1] + self.fRec10[1] + fRec35 + fRec31 + fTemp19);
            self.fRec9[0] =
                self.fRec34[1] + self.fRec30[1] + self.fRec22[1] + self.fRec14[1] + fRec35 + fRec31 + fTemp20
                    - (self.fRec38[1] + self.fRec26[1] + self.fRec18[1] + self.fRec10[1] + fRec39 + fRec27 + fTemp19);
            let mut fTemp21: F32 = 0.37 * (self.fRec3[0] + self.fRec4[0]);
            let mut fTemp22: F32 = fTemp21 + fTemp1;
            self.fRec1[0] = fTemp22 - fSlow8 * self.fRec1[2];
            let mut fTemp23: F32 = fSlow8 * self.fRec1[0];
            let mut fTemp24: F32 = fSlow5 * (self.fRec1[2] + fTemp23 - fTemp22);
            let mut fTemp25: F32 = fTemp23 + fTemp21 + self.fRec1[2];
            self.fRec0[0] = 0.5 * (fTemp25 - fTemp1 + fTemp24) + fTemp0 - fSlow3 * self.fRec0[2];
            let mut fTemp26: F32 = 0.5 * (fTemp25 + fTemp24 - fTemp1);
            let mut fTemp27: F32 = self.fRec0[2] + fSlow3 * self.fRec0[0];
            self.fRec42[0] = fSlow89 + self.fConst45 * self.fRec42[1];
            let mut fTemp28: F32 = self.fRec42[0] + 1.0;
            let mut fTemp29: F32 = 1.0 - 0.5 * fTemp28;
            self.fRec43[0] = fSlow90 + self.fConst45 * self.fRec43[1];
            *output0 = 0.5
                * self.fRec43[0]
                * (fTemp2 * fTemp28 + fTemp29 * (fTemp27 + fTemp26 + fSlow0 * (fTemp27 - (fTemp0 + fTemp26)) - fTemp0));
            let mut fTemp30: F32 = fSlow4 * self.fRec44[1];
            let mut fTemp31: F32 = fSlow9 * self.fRec45[1];
            let mut fTemp32: F32 = 0.37 * (self.fRec3[0] - self.fRec4[0]);
            let mut fTemp33: F32 = fTemp32 + fTemp31;
            self.fRec45[0] = fTemp33 - fSlow8 * self.fRec45[2];
            let mut fTemp34: F32 = fSlow8 * self.fRec45[0];
            let mut fTemp35: F32 = fSlow5 * (self.fRec45[2] + fTemp34 - fTemp33);
            let mut fTemp36: F32 = fTemp34 + fTemp32 + self.fRec45[2];
            self.fRec44[0] = 0.5 * (fTemp36 - fTemp31 + fTemp35) + fTemp30 - fSlow3 * self.fRec44[2];
            let mut fTemp37: F32 = 0.5 * (fTemp36 + fTemp35 - fTemp31);
            let mut fTemp38: F32 = self.fRec44[2] + fSlow3 * self.fRec44[0];
            *output1 = 0.5
                * self.fRec43[0]
                * (fTemp10 * fTemp28
                    + fTemp29 * (fTemp38 + fTemp37 + fSlow0 * (fTemp38 - (fTemp30 + fTemp37)) - fTemp30));
            self.fRec13[1] = self.fRec13[0];
            self.fRec12[1] = self.fRec12[0];
            self.IOTA0 = i32::wrapping_add(self.IOTA0, 1);
            self.fRec10[1] = self.fRec10[0];
            self.fRec17[1] = self.fRec17[0];
            self.fRec16[1] = self.fRec16[0];
            self.fRec14[1] = self.fRec14[0];
            self.fRec21[1] = self.fRec21[0];
            self.fRec20[1] = self.fRec20[0];
            self.fRec18[1] = self.fRec18[0];
            self.fRec25[1] = self.fRec25[0];
            self.fRec24[1] = self.fRec24[0];
            self.fRec22[1] = self.fRec22[0];
            self.fRec29[1] = self.fRec29[0];
            self.fRec28[1] = self.fRec28[0];
            self.fRec26[1] = self.fRec26[0];
            self.fRec33[1] = self.fRec33[0];
            self.fRec32[1] = self.fRec32[0];
            self.fRec30[1] = self.fRec30[0];
            self.fRec37[1] = self.fRec37[0];
            self.fRec36[1] = self.fRec36[0];
            self.fRec34[1] = self.fRec34[0];
            self.fRec41[1] = self.fRec41[0];
            self.fRec40[1] = self.fRec40[0];
            self.fRec38[1] = self.fRec38[0];
            self.fRec2[2] = self.fRec2[1];
            self.fRec2[1] = self.fRec2[0];
            self.fRec3[2] = self.fRec3[1];
            self.fRec3[1] = self.fRec3[0];
            self.fRec4[2] = self.fRec4[1];
            self.fRec4[1] = self.fRec4[0];
            self.fRec5[2] = self.fRec5[1];
            self.fRec5[1] = self.fRec5[0];
            self.fRec6[2] = self.fRec6[1];
            self.fRec6[1] = self.fRec6[0];
            self.fRec7[2] = self.fRec7[1];
            self.fRec7[1] = self.fRec7[0];
            self.fRec8[2] = self.fRec8[1];
            self.fRec8[1] = self.fRec8[0];
            self.fRec9[2] = self.fRec9[1];
            self.fRec9[1] = self.fRec9[0];
            self.fRec1[2] = self.fRec1[1];
            self.fRec1[1] = self.fRec1[0];
            self.fRec0[2] = self.fRec0[1];
            self.fRec0[1] = self.fRec0[0];
            self.fRec42[1] = self.fRec42[0];
            self.fRec43[1] = self.fRec43[0];
            self.fRec45[2] = self.fRec45[1];
            self.fRec45[1] = self.fRec45[0];
            self.fRec44[2] = self.fRec44[1];
            self.fRec44[1] = self.fRec44[0];
        }
    }
}
