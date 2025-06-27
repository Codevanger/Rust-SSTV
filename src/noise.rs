use crate::envelope::EnvelopeKind;
use anyhow::Result;
use rand::rng;
use rand_distr::{Distribution, Normal};

#[derive(Clone, Debug)]
pub struct NoiseParams {
    pub level: u8,
    pub env: EnvelopeKind,
    pub repeat: f32,
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            level: 0,
            env: EnvelopeKind::Const,
            repeat: 1.0,
        }
    }
}

pub struct NoiseProcessor {
    pub params: NoiseParams,
}

impl NoiseProcessor {
    pub fn new() -> Self {
        Self {
            params: NoiseParams::default(),
        }
    }

    pub fn new_with_params(params: NoiseParams) -> Self {
        Self { params }
    }

    /// Применяет AWGN шум к массиву сэмплов
    pub fn apply_noise(&self, samples: &mut [f32]) -> Result<()> {
        if self.params.level == 0 {
            return Ok(());
        }

        let len = samples.len();

        // Рассчитываем SNR и параметры шума
        let snr_db = 30.0 * (1.0 - f32::from(self.params.level) / 100.0) + 0.1;
        let rms_sig = self.calculate_rms_signal(samples);
        let rms_noise = rms_sig / 10f32.powf(snr_db / 20.0);

        // Создаем генератор нормального распределения
        let normal = Normal::new(0.0, rms_noise).unwrap();
        let mut rng = rng();

        // Применяем шум с огибающей
        for (i, sample) in samples.iter_mut().enumerate() {
            let env_factor = self.params.env.factor(i, len, self.params.repeat);
            let noise_value = normal.sample(&mut rng) * env_factor;
            *sample = (*sample + noise_value).clamp(-1.0, 1.0);
        }

        Ok(())
    }

    /// Рассчитывает RMS уровень сигнала
    fn calculate_rms_signal(&self, samples: &[f32]) -> f32 {
        let sum_squares: f32 = samples.iter().map(|x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Рассчитывает SNR для заданного уровня шума
    pub fn calculate_snr_db(&self) -> f32 {
        if self.params.level == 0 {
            return f32::INFINITY;
        }
        30.0 * (1.0 - f32::from(self.params.level) / 100.0) + 0.1
    }

    /// Устанавливает уровень шума
    pub fn set_level(&mut self, level: u8) {
        self.params.level = level.min(100);
    }

    /// Устанавливает тип огибающей
    pub fn set_envelope(&mut self, env: EnvelopeKind) {
        self.params.env = env;
    }

    /// Устанавливает коэффициент повторения огибающей
    pub fn set_repeat(&mut self, repeat: f32) {
        self.params.repeat = repeat.max(0.1);
    }

    /// Проверяет, включен ли шум
    pub fn is_enabled(&self) -> bool {
        self.params.level > 0
    }

    /// Возвращает описание текущих настроек шума
    pub fn describe(&self) -> String {
        if !self.is_enabled() {
            return "Шум отключен".to_string();
        }

        format!(
            "Шум: {}%, огибающая: {}, повторение: {:.1}x, SNR: {:.1} дБ",
            self.params.level,
            self.params.env.name(),
            self.params.repeat,
            self.calculate_snr_db()
        )
    }
}
