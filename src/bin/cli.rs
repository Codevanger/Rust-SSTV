use anyhow::{Context, Result};
use clap::Parser;
use image::ImageReader;
use sstv_processor::{EnvelopeKind, NoiseParams, ProcessingParams, RetarderParams, SSTVProcessor};

/// Параметры CLI
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Главная картинка PNG/JPG
    #[arg(short = 'i', long)]
    input: String,

    /// Итоговый файл
    #[arg(short = 'o', long, default_value = "output.png")]
    output: String,

    // ── Шум ──────────────────────────────────────────────────
    /// Уровень шума 0–100
    #[arg(short = 'n', long, default_value_t = 0u8,
          value_parser = clap::value_parser!(u8).range(0..=100))]
    noise: u8,

    /// Огибающая шума
    #[arg(long, default_value = "const")]
    noise_env: EnvelopeKind,

    /// Коэффициент повторения огибающей шума
    #[arg(long, default_value_t = 1.0)]
    noise_repeat: f32,

    // ── Ретардер ─────────────────────────────────────────────
    /// Картинка-"призрак"
    #[arg(short = 'r', long)]
    retarder: Option<String>,

    /// Уровень ретардера 0–1
    #[arg(long, default_value_t = 0.3)]
    level: f32,

    /// Огибающая ретардера
    #[arg(long, default_value = "const")]
    ret_env: EnvelopeKind,

    /// Коэффициент повторения ретардера
    #[arg(long, default_value_t = 1.0)]
    ret_repeat: f32,

    /// Задержка ретардера, мс
    #[arg(long, default_value_t = 0u32)]
    delay_ms: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Читаем исходную картинку
    let main_image = ImageReader::open(&args.input)?
        .decode()
        .with_context(|| "Не смог декодировать исходное изображение")?;

    // Читаем ретардер, если указан
    let retarder_image = if let Some(path) = &args.retarder {
        Some(
            ImageReader::open(path)?
                .decode()
                .with_context(|| "Не смог декодировать ретардер-картинку")?,
        )
    } else {
        None
    };

    // Настраиваем параметры
    let noise_params = NoiseParams {
        level: args.noise,
        env: args.noise_env,
        repeat: args.noise_repeat,
    };

    let retarder_params = RetarderParams {
        level: args.level,
        env: args.ret_env,
        repeat: args.ret_repeat,
        delay_ms: args.delay_ms,
    };

    let params = ProcessingParams {
        noise: noise_params,
        retarder: retarder_params,
    };

    // Обрабатываем
    let mut processor = SSTVProcessor::new_with_params(params);

    let result = processor.process(&main_image, retarder_image.as_ref())?;

    // Сохраняем
    processor.save_result(&result, &args.output)?;

    println!("Готово: {}", args.output);
    Ok(())
}
