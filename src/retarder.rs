use crate::envelope::EnvelopeKind;
use anyhow::Result;
use image::DynamicImage;
use rsstv::{common::SSTVMode, martinm1::MartinM1};

#[derive(Clone, Debug)]
pub struct RetarderParams {
    pub level: f32,
    pub env: EnvelopeKind,
    pub repeat: f32,
    pub delay_ms: u32,
}

impl Default for RetarderParams {
    fn default() -> Self {
        Self {
            level: 0.3,
            env: EnvelopeKind::Const,
            repeat: 1.0,
            delay_ms: 0,
        }
    }
}

pub struct RetarderProcessor {
    pub params: RetarderParams,
}

impl RetarderProcessor {
    pub fn new() -> Self {
        Self {
            params: RetarderParams::default(),
        }
    }

    pub fn new_with_params(params: RetarderParams) -> Self {
        Self { params }
    }

    /// Применяет эффект ретардера (призрака) к массиву сэмплов
    pub fn apply_retarder(&self, samples: &mut [f32], retarder_image: &DynamicImage) -> Result<()> {
        if self.params.level <= 0.0 {
            return Ok(());
        }

        // Кодируем ретардер в сэмплы
        let mut retarder_samples = self.encode_retarder_image(retarder_image)?;
        let samples_len = samples.len();

        // Применяем задержку
        self.apply_delay(&mut retarder_samples);

        // Подгоняем длину ретардера
        self.adjust_retarder_length(&mut retarder_samples, samples_len);

        // Микшируем с основным сигналом
        self.mix_retarder(samples, &retarder_samples);

        Ok(())
    }

    /// Кодирует изображение ретардера в SSTV сэмплы
    fn encode_retarder_image(&self, retarder_image: &DynamicImage) -> Result<Vec<f32>> {
        let mut encoder = MartinM1::new();
        let samples = encoder.encode(retarder_image.clone()).to_samples();
        Ok(samples)
    }

    /// Применяет задержку к сэмплам ретардера
    fn apply_delay(&self, retarder_samples: &mut Vec<f32>) {
        if self.params.delay_ms > 0 {
            let delay_samples = (self.params.delay_ms as f32 / 1000.0 * 11_025.0) as usize;
            if delay_samples > 0 {
                retarder_samples.splice(0..0, std::iter::repeat(0.0).take(delay_samples));
            }
        }
    }

    /// Подгоняет длину массива ретардера под основной сигнал
    fn adjust_retarder_length(&self, retarder_samples: &mut Vec<f32>, target_length: usize) {
        if retarder_samples.len() < target_length {
            let padding = target_length - retarder_samples.len();
            retarder_samples.extend(std::iter::repeat(0.0).take(padding));
        }
    }

    /// Микширует ретардер с основным сигналом
    fn mix_retarder(&self, main_samples: &mut [f32], retarder_samples: &[f32]) {
        let level = self.params.level.clamp(0.0, 1.0);
        let main_len = main_samples.len();
        let retarder_len = retarder_samples.len();

        for (i, main_sample) in main_samples.iter_mut().enumerate() {
            // Позиция в ретардере с учётом коэффициента повторения
            let retarder_idx = ((i as f32) * self.params.repeat) as usize;
            let retarder_value = retarder_samples[retarder_idx % retarder_len];

            // Применяем огибающую
            let env_factor = self.params.env.factor(i, main_len, self.params.repeat);

            // Микшируем
            *main_sample = (*main_sample + retarder_value * level * env_factor).clamp(-1.0, 1.0);
        }
    }

    /// Устанавливает уровень ретардера
    pub fn set_level(&mut self, level: f32) {
        self.params.level = level.clamp(0.0, 1.0);
    }

    /// Устанавливает тип огибающей
    pub fn set_envelope(&mut self, env: EnvelopeKind) {
        self.params.env = env;
    }

    /// Устанавливает коэффициент повторения
    pub fn set_repeat(&mut self, repeat: f32) {
        self.params.repeat = repeat.max(0.1);
    }

    /// Устанавливает задержку в миллисекундах
    pub fn set_delay_ms(&mut self, delay_ms: u32) {
        self.params.delay_ms = delay_ms;
    }

    /// Проверяет, включен ли ретардер
    pub fn is_enabled(&self) -> bool {
        self.params.level > 0.0
    }

    /// Рассчитывает задержку в сэмплах
    pub fn get_delay_samples(&self) -> usize {
        (self.params.delay_ms as f32 / 1000.0 * 11_025.0) as usize
    }

    /// Возвращает описание текущих настроек ретардера
    pub fn describe(&self) -> String {
        if !self.is_enabled() {
            return "Ретардер отключен".to_string();
        }

        format!(
            "Ретардер: {:.1}%, огибающая: {}, повторение: {:.1}x, задержка: {} мс",
            self.params.level * 100.0,
            self.params.env.name(),
            self.params.repeat,
            self.params.delay_ms
        )
    }

    /// Предварительная оценка влияния ретардера на длину сигнала
    pub fn estimate_output_length(&self, base_length: usize) -> usize {
        let delay_samples = self.get_delay_samples();
        base_length + delay_samples
    }
}
