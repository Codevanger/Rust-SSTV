use crate::noise::{NoiseParams, NoiseProcessor};
use crate::retarder::{RetarderParams, RetarderProcessor};
use anyhow::{Context, Result, bail};
use hound::{SampleFormat, WavSpec, WavWriter};
use image::{DynamicImage, ImageFormat};
use rsstv::SAMPLE_RATE;
use rsstv::{
    common::{DecodeResult, SSTVMode},
    martinm1::MartinM1,
};

#[derive(Clone, Debug)]
pub struct ProcessingParams {
    pub noise: NoiseParams,
    pub retarder: RetarderParams,
}

impl Default for ProcessingParams {
    fn default() -> Self {
        Self {
            noise: NoiseParams::default(),
            retarder: RetarderParams::default(),
        }
    }
}

pub struct SSTVProcessor {
    pub params: ProcessingParams,
    noise_processor: NoiseProcessor,
    retarder_processor: RetarderProcessor,
}

impl SSTVProcessor {
    pub fn new() -> Self {
        Self {
            params: ProcessingParams::default(),
            noise_processor: NoiseProcessor::new(),
            retarder_processor: RetarderProcessor::new(),
        }
    }

    pub fn new_with_params(params: ProcessingParams) -> Self {
        Self {
            noise_processor: NoiseProcessor::new_with_params(params.noise.clone()),
            retarder_processor: RetarderProcessor::new_with_params(params.retarder.clone()),
            params,
        }
    }

    pub fn process(
        &mut self,
        main_image: &DynamicImage,
        retarder_image: Option<&DynamicImage>,
    ) -> Result<DynamicImage> {
        self.noise_processor.params = self.params.noise.clone();
        self.retarder_processor.params = self.params.retarder.clone();

        let mut enc_main = MartinM1::new();
        let mut samples = enc_main.encode(main_image.clone()).to_samples();

        if let Some(retarder) = retarder_image {
            self.retarder_processor
                .apply_retarder(&mut samples, retarder)?;
        }

        self.noise_processor.apply_noise(&mut samples)?;

        let spec = WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE as u32, // 44 100 Гц :contentReference[oaicite:0]{index=0}
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create("debug.wav", spec)?;

        let max = i16::MAX as f32;
        for s in &samples {
            writer.write_sample((s.clamp(-1.0, 1.0) * max) as i16)?;
        }
        writer.finalize()?;

        let mut dec = MartinM1::new();
        let decoded = match dec.decode(&samples) {
            DecodeResult::Finished(img) | DecodeResult::Partial(img) => img,
            DecodeResult::NoneFound => {
                bail!("Декодер не нашёл изображение — уменьшите Шум или Ретардер")
            }
        };

        Ok(decoded)
    }

    pub fn noise_processor(&self) -> &NoiseProcessor {
        &self.noise_processor
    }

    pub fn noise_processor_mut(&mut self) -> &mut NoiseProcessor {
        &mut self.noise_processor
    }

    pub fn retarder_processor(&self) -> &RetarderProcessor {
        &self.retarder_processor
    }

    pub fn retarder_processor_mut(&mut self) -> &mut RetarderProcessor {
        &mut self.retarder_processor
    }

    pub fn update_noise_params(&mut self, params: NoiseParams) {
        self.params.noise = params.clone();
        self.noise_processor.params = params;
    }

    pub fn update_retarder_params(&mut self, params: RetarderParams) {
        self.params.retarder = params.clone();
        self.retarder_processor.params = params;
    }

    pub fn save_result(&self, result: &DynamicImage, output_path: &str) -> Result<()> {
        result
            .save_with_format(output_path, ImageFormat::Png)
            .with_context(|| format!("Не удалось сохранить результат в {}", output_path))
    }

    pub fn describe(&self) -> String {
        format!(
            "SSTV Processor:\n{}\n{}",
            self.noise_processor.describe(),
            self.retarder_processor.describe()
        )
    }

    /// Проверяет, включены ли какие-либо эффекты
    pub fn has_effects(&self) -> bool {
        self.noise_processor.is_enabled() || self.retarder_processor.is_enabled()
    }
}
