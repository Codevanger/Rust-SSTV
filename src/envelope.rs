use rand::random;
use std::f32::consts::PI;

/// Огибающие для уровня во времени
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EnvelopeKind {
    Const,
    Sin,
    Tri,
    Saw,
    Square,
    Rand,
}

impl EnvelopeKind {
    /// Вернёт коэффициент (0…1) для сэмпла `idx` при длине `len` и коэффициенте повторения `rep`
    pub fn factor(self, idx: usize, len: usize, rep: f32) -> f32 {
        let t = idx as f32 / (len as f32 - 1.0); // 0‥1
        let x = t * rep;
        match self {
            Self::Const => 1.0,
            Self::Sin => 0.5 * (1.0 + (2.0 * PI * x).sin()),
            Self::Tri => 1.0 - (2.0 * (x.fract()) - 1.0).abs(),
            Self::Saw => x.fract(),
            Self::Square => {
                if (2.0 * PI * x).sin() >= 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Rand => random::<f32>(),
        }
    }

    pub const ALL: &'static [EnvelopeKind] = &[
        EnvelopeKind::Const,
        EnvelopeKind::Sin,
        EnvelopeKind::Tri,
        EnvelopeKind::Saw,
        EnvelopeKind::Square,
        EnvelopeKind::Rand,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            EnvelopeKind::Const => "Const",
            EnvelopeKind::Sin => "Sin",
            EnvelopeKind::Tri => "Tri",
            EnvelopeKind::Saw => "Saw",
            EnvelopeKind::Square => "Square",
            EnvelopeKind::Rand => "Rand",
        }
    }
}

impl clap::ValueEnum for EnvelopeKind {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Const,
            Self::Sin,
            Self::Tri,
            Self::Saw,
            Self::Square,
            Self::Rand,
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Const => clap::builder::PossibleValue::new("const"),
            Self::Sin => clap::builder::PossibleValue::new("sin"),
            Self::Tri => clap::builder::PossibleValue::new("tri"),
            Self::Saw => clap::builder::PossibleValue::new("saw"),
            Self::Square => clap::builder::PossibleValue::new("square"),
            Self::Rand => clap::builder::PossibleValue::new("rand"),
        })
    }
}
