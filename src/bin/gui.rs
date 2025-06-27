use egui::{ColorImage, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Импортируем из локального крейта
use sstv_processor::{EnvelopeKind, SSTVProcessor};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("SSTV Processor GUI"),
        ..Default::default()
    };

    eframe::run_native(
        "SSTV Processor",
        options,
        Box::new(|cc| Box::new(SSTVApp::new(cc))),
    )
}

struct SSTVApp {
    processor: SSTVProcessor,
    main_image: Option<DynamicImage>,
    retarder_image: Option<DynamicImage>,
    result_image: Option<DynamicImage>,

    main_texture: Option<TextureHandle>,
    retarder_texture: Option<TextureHandle>,
    result_texture: Option<TextureHandle>,

    processing: Arc<Mutex<bool>>,
    last_process_time: Option<Instant>,
    auto_process: bool,
    manual_processing_requested: bool,

    // Параметры из интерфейса
    main_image_path: String,
    retarder_image_path: String,
    output_path: String,

    // Локальные копии параметров для GUI
    noise_level: u8,
    noise_env: EnvelopeKind,
    noise_repeat: f32,
    retarder_level: f32,
    retarder_env: EnvelopeKind,
    retarder_repeat: f32,
    delay_ms: u32,
}

impl SSTVApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            processor: SSTVProcessor::new(),
            main_image: None,
            retarder_image: None,
            result_image: None,
            main_texture: None,
            retarder_texture: None,
            result_texture: None,
            processing: Arc::new(Mutex::new(false)),
            last_process_time: None,
            auto_process: false, // Отключаем по умолчанию
            manual_processing_requested: false,
            main_image_path: String::new(),
            retarder_image_path: String::new(),
            output_path: "output.png".to_string(),

            // Инициализируем локальные параметры значениями по умолчанию
            noise_level: 0,
            noise_env: EnvelopeKind::Const,
            noise_repeat: 1.0,
            retarder_level: 0.3,
            retarder_env: EnvelopeKind::Const,
            retarder_repeat: 1.0,
            delay_ms: 0,
        }
    }

    fn load_image(&mut self, path: &str, is_main: bool) {
        println!("Загружаем изображение: {}", path);
        match image::open(path) {
            Ok(img) => {
                let (w, h) = img.dimensions();
                println!("Загружено: {}×{} пикселей", w, h);

                if is_main {
                    self.main_image = Some(img);
                } else {
                    self.retarder_image = Some(img);
                }
                self.schedule_processing();
            }
            Err(e) => {
                eprintln!("Не удалось загрузить изображение {}: {}", path, e);
            }
        }
    }

    fn load_image_and_update_texture(&mut self, ctx: &egui::Context, path: &str, is_main: bool) {
        self.load_image(path, is_main);

        // Создаем текстуры после загрузки изображений
        if is_main && self.main_image.is_some() {
            let img = self.main_image.as_ref().unwrap();
            let color_image = Self::dynamic_image_to_color_image(img);
            let handle = ctx.load_texture("main", color_image, egui::TextureOptions::LINEAR);
            self.main_texture = Some(handle);
        } else if !is_main && self.retarder_image.is_some() {
            let img = self.retarder_image.as_ref().unwrap();
            let color_image = Self::dynamic_image_to_color_image(img);
            let handle = ctx.load_texture("retarder", color_image, egui::TextureOptions::LINEAR);
            self.retarder_texture = Some(handle);
        }
    }

    fn prepare_image_for_sstv(img: &DynamicImage) -> DynamicImage {
        let (w, h) = img.dimensions();

        // SSTV Martin M1 использует 320×256
        const SSTV_WIDTH: u32 = 320;
        const SSTV_HEIGHT: u32 = 256;

        if w != SSTV_WIDTH || h != SSTV_HEIGHT {
            println!(
                "Изменяем размер с {}×{} на {}×{} для SSTV",
                w, h, SSTV_WIDTH, SSTV_HEIGHT
            );
            img.resize_exact(
                SSTV_WIDTH,
                SSTV_HEIGHT,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img.clone()
        }
    }

    fn dynamic_image_to_color_image(img: &DynamicImage) -> ColorImage {
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        ColorImage { size, pixels }
    }

    fn schedule_processing(&mut self) {
        if self.auto_process {
            self.last_process_time = Some(Instant::now());
        }
    }

    fn request_manual_processing(&mut self) {
        self.manual_processing_requested = true;
    }

    fn sync_params_to_processor(&mut self) {
        // Обновляем параметры без пересоздания процессоров
        self.processor.params.noise.level = self.noise_level;
        self.processor.params.noise.env = self.noise_env;
        self.processor.params.noise.repeat = self.noise_repeat;

        self.processor.params.retarder.level = self.retarder_level;
        self.processor.params.retarder.env = self.retarder_env;
        self.processor.params.retarder.repeat = self.retarder_repeat;
        self.processor.params.retarder.delay_ms = self.delay_ms;
    }

    fn process_if_needed(&mut self, ctx: &egui::Context) {
        let should_process = if self.manual_processing_requested {
            self.manual_processing_requested = false;
            true
        } else if let Some(last_time) = self.last_process_time {
            if last_time.elapsed().as_millis() > 2000 {
                // Увеличиваем debounce до 2 секунд
                self.last_process_time = None;
                true
            } else {
                false
            }
        } else {
            false
        };

        if should_process {
            self.process_images(ctx);
        }
    }

    fn process_images(&mut self, ctx: &egui::Context) {
        // Проверяем, что не происходит обработка и есть главное изображение
        if self.processing.try_lock().map_or(true, |p| *p) {
            return;
        }

        let (original_main_image, _original_dimensions) = match &self.main_image {
            Some(img) => {
                let dims = img.dimensions();
                (img.clone(), dims)
            }
            None => return,
        };

        // Подготавливаем изображения для SSTV (320×256)
        let main_image = Self::prepare_image_for_sstv(&original_main_image);
        let retarder_image = self
            .retarder_image
            .as_ref()
            .map(|img| Self::prepare_image_for_sstv(img));

        // Устанавливаем флаг обработки
        if let Ok(mut is_processing) = self.processing.try_lock() {
            *is_processing = true;
        }

        // Синхронизируем параметры ТОЛЬКО ОДИН РАЗ
        self.sync_params_to_processor();

        println!("Начинаем SSTV обработку (320×256)...");
        let start_time = std::time::Instant::now();

        // Выполняем обработку на уменьшенных изображениях
        match self.processor.process(&main_image, retarder_image.as_ref()) {
            Ok(sstv_result) => {
                let duration = start_time.elapsed();
                println!("Обработка завершена за {:.2}с", duration.as_secs_f32());
                let final_result = sstv_result;

                let color_image = Self::dynamic_image_to_color_image(&final_result);
                let handle = ctx.load_texture("result", color_image, egui::TextureOptions::LINEAR);
                self.result_image = Some(final_result);
                self.result_texture = Some(handle);
            }
            Err(e) => {
                let duration = start_time.elapsed();
                eprintln!("Ошибка обработки за {:.2}с: {}", duration.as_secs_f32(), e);
            }
        }

        // Сбрасываем флаг обработки
        if let Ok(mut is_processing) = self.processing.try_lock() {
            *is_processing = false;
        }
    }

    fn save_result(&self) {
        if let Some(result) = &self.result_image {
            match self.processor.save_result(result, &self.output_path) {
                Ok(_) => {
                    println!("Сохранено в: {}", self.output_path);

                    // Показываем системное уведомление об успешном сохранении
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Err(e) = std::process::Command::new("notify-send")
                            .args(&[
                                "SSTV Processor",
                                &format!("Файл сохранен: {}", self.output_path),
                            ])
                            .output()
                        {
                            // Если notify-send не работает, просто игнорируем
                            let _ = e;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка сохранения: {}", e);
                }
            }
        } else {
            eprintln!("Нет результата для сохранения");
        }
    }
}

impl eframe::App for SSTVApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Проверяем, нужно ли обработать изображения
        self.process_if_needed(ctx);

        // Левая панель - параметры
        egui::SidePanel::left("parameters").show(ctx, |ui| {
            ui.heading("Параметры SSTV");

            ui.separator();
            ui.label("Файлы:");

            ui.horizontal(|ui| {
                ui
                    .label("Основное изображение:")
                    .on_hover_text("Изображение для обработки");

                if ui.button("Выбрать").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Изображения", &["png", "jpg", "jpeg", "bmp", "gif", "tiff", "webp"])
                        .set_title("Выберите основное изображение")
                        .pick_file()
                    {
                        self.main_image_path = path.to_string_lossy().to_string();
                        let path_clone = self.main_image_path.clone();
                        self.load_image_and_update_texture(ctx, &path_clone, true);
                    }
                }
            });
            ui.text_edit_singleline(&mut self.main_image_path);
            if ui.button("Загрузить по пути").clicked() && !self.main_image_path.is_empty() {
                let path = self.main_image_path.clone();
                self.load_image_and_update_texture(ctx, &path, true);
                self.request_manual_processing();  // Обрабатываем сразу при загрузке
                self.request_manual_processing();  // Обрабатываем сразу при загрузке
            }

            ui.horizontal(|ui| {
                ui.label("Ретардер:");
                if ui.button("Выбрать").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Изображения", &["png", "jpg", "jpeg", "bmp", "gif", "tiff", "webp"])
                        .set_title("Выберите изображение ретардера")
                        .pick_file()
                    {
                        self.retarder_image_path = path.to_string_lossy().to_string();
                        let path_clone = self.retarder_image_path.clone();
                        self.load_image_and_update_texture(ctx, &path_clone, false);
                        self.request_manual_processing();  // Обрабатываем сразу при загрузке
                    }
                }
                if ui.button("Очистить").clicked() {
                    self.retarder_image = None;
                    self.retarder_texture = None;
                    self.retarder_image_path.clear();
                    self.schedule_processing();
                }
            });
            ui.text_edit_singleline(&mut self.retarder_image_path);
            if ui.button("Загрузить по пути").clicked() && !self.retarder_image_path.is_empty() {
                let path = self.retarder_image_path.clone();
                self.load_image_and_update_texture(ctx, &path, false);
                self.request_manual_processing();  // Обрабатываем сразу при загрузке
            }

            ui.separator();
            ui.label("Шум:");

            if ui.add(egui::Slider::new(&mut self.noise_level, 0..=100)
                .text("Уровень")).changed() {
                self.schedule_processing();
            }

            egui::ComboBox::from_label("Огибающая шума")
                .selected_text(self.noise_env.name())
                .show_ui(ui, |ui| {
                    for &env in EnvelopeKind::ALL {
                        if ui.selectable_value(&mut self.noise_env, env, env.name()).changed() {
                            self.schedule_processing();
                        }
                    }
                });

            if ui.add(egui::Slider::new(&mut self.noise_repeat, 0.1..=10.0)
                .text("Повторение")).changed() {
                self.schedule_processing();
            }

            ui.separator();
            ui.label("Ретардер:");

            if ui.add(egui::Slider::new(&mut self.retarder_level, 0.0..=1.0)
                .text("Уровень")).changed() {
                self.schedule_processing();
            }

            egui::ComboBox::from_label("Огибающая ретардера")
                .selected_text(self.retarder_env.name())
                .show_ui(ui, |ui| {
                    for &env in EnvelopeKind::ALL {
                        if ui.selectable_value(&mut self.retarder_env, env, env.name()).changed() {
                            self.schedule_processing();
                        }
                    }
                });

            if ui.add(egui::Slider::new(&mut self.retarder_repeat, 0.1..=10.0)
                .text("Повторение")).changed() {
                self.schedule_processing();
            }

            if ui.add(egui::Slider::new(&mut self.delay_ms, 0..=1000)
                .text("Задержка (мс)")).changed() {
                self.schedule_processing();
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.auto_process, "Автообработка (медленно!)");
                if ui.button("🔄 Обработать сейчас").clicked() {
                    self.request_manual_processing();
                }
            });

            if !self.auto_process {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 165, 0),
                    "💡 Автообработка отключена. Нажмите '🔄 Обработать сейчас' для получения результата."
                );
            }

            // Показываем статус обработки
            if self.processing.try_lock().map_or(false, |p| *p) {
                ui.colored_label(egui::Color32::YELLOW, "⏳ Обработка...");
            } else if self.last_process_time.is_some() {
                let remaining = self.last_process_time.map_or(0, |t| {
                    2000_u128.saturating_sub(t.elapsed().as_millis())
                });
                if remaining > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 150, 255),
                        format!("⏱️ Автообработка через {:.1}с", remaining as f32 / 1000.0)
                    );
                }
            }

            ui.separator();

            // Информация о текущих настройках
            ui.collapsing("Информация", |ui| {
                ui.label(format!("Шум: {}%", self.noise_level));
                if self.noise_level > 0 {
                    let snr = 30.0 * (1.0 - self.noise_level as f32 / 100.0) + 0.1;
                    ui.label(format!("SNR: {:.1} дБ", snr));
                }
                ui.label(format!("Ретардер: {:.1}%", self.retarder_level * 100.0));
                if self.delay_ms > 0 {
                    ui.label(format!("Задержка: {} мс", self.delay_ms));
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Выходной файл:");
                ui.text_edit_singleline(&mut self.output_path);
                if ui.button("Выбрать папку").clicked() {
                    if let Some(folder) = rfd::FileDialog::new()
                        .set_title("Выберите папку для сохранения")
                        .pick_folder()
                    {
                        let file_name = std::path::Path::new(&self.output_path)
                            .file_name()
                            .unwrap_or(std::ffi::OsStr::new("output.png"))
                            .to_string_lossy()
                            .to_string();
                        self.output_path = folder.join(&file_name).to_string_lossy().to_string();
                    }
                }
            });

            if ui.button("💾 Сохранить результат").clicked() {
                if self.result_image.is_some() {
                    self.save_result();
                } else {
                    eprintln!("Нет результата для сохранения. Сначала обработайте изображение.");
                }
            }
        });

        // Центральная область - изображения
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Предварительный просмотр");

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Исходное изображение
                    ui.vertical(|ui| {
                        ui.label("Исходное");
                        if let Some(texture) = &self.main_texture {
                            let size = texture.size_vec2();
                            let scale = (300.0 / size.x.max(size.y)).min(1.0);
                            let scaled_size = size * scale;
                            ui.add(
                                egui::Image::from_texture(texture).fit_to_exact_size(scaled_size),
                            );
                            ui.label(format!("{}×{}", size.x as u32, size.y as u32));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Изображение не загружено");
                            ui.allocate_space(Vec2::new(300.0, 200.0));
                        }
                    });

                    ui.separator();

                    // Ретардер
                    ui.vertical(|ui| {
                        ui.label("Ретардер");
                        if let Some(texture) = &self.retarder_texture {
                            let size = texture.size_vec2();
                            let scale = (300.0 / size.x.max(size.y)).min(1.0);
                            let scaled_size = size * scale;
                            ui.add(
                                egui::Image::from_texture(texture).fit_to_exact_size(scaled_size),
                            );
                            ui.label(format!("{}×{}", size.x as u32, size.y as u32));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Ретардер не загружен");
                            ui.allocate_space(Vec2::new(300.0, 200.0));
                        }
                    });

                    ui.separator();

                    // Результат
                    ui.vertical(|ui| {
                        ui.label("Результат");
                        if let Some(texture) = &self.result_texture {
                            let size = texture.size_vec2();
                            let scale = (300.0 / size.x.max(size.y)).min(1.0);
                            let scaled_size = size * scale;
                            ui.add(
                                egui::Image::from_texture(texture).fit_to_exact_size(scaled_size),
                            );
                            ui.label(format!("{}×{}", size.x as u32, size.y as u32));
                        } else if self.processing.try_lock().map_or(false, |p| *p) {
                            ui.colored_label(egui::Color32::YELLOW, "Обработка...");
                            ui.allocate_space(Vec2::new(300.0, 200.0));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Результат не готов");
                            ui.allocate_space(Vec2::new(300.0, 200.0));
                        }
                    });
                });

                ui.separator();

                // Дополнительная информация
                if self.main_image.is_some() || self.result_image.is_some() {
                    ui.collapsing("📊 Детали обработки", |ui| {
                        if let Some(main_img) = &self.main_image {
                            let (w, h) = main_img.dimensions();
                            ui.label(format!("Исходное разрешение: {}×{}", w, h));

                            if w != 320 || h != 256 {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 165, 0),
                                    "⚠️ Изображение будет сжато до 320×256 для SSTV обработки",
                                );
                            } else {
                                ui.colored_label(
                                    egui::Color32::from_rgb(0, 255, 0),
                                    "✅ Идеальный размер для SSTV (320×256)",
                                );
                            }
                        }

                        ui.label("SSTV режим: Martin M1 (320×256)");
                        ui.label("Длительность передачи: ~114.5 секунд");

                        if self.noise_level > 0 {
                            let snr = 30.0 * (1.0 - self.noise_level as f32 / 100.0) + 0.1;
                            ui.label(format!("Отношение сигнал/шум: {:.1} дБ", snr));
                        }

                        if self.delay_ms > 0 {
                            let delay_samples = (self.delay_ms as f32 / 1000.0 * 11_025.0) as usize;
                            ui.label(format!("Задержка в сэмплах: {}", delay_samples));
                        }
                    });
                }
            });
        });

        // Запрашиваем перерисовку для анимации
        if self.processing.try_lock().map_or(false, |p| *p) || self.last_process_time.is_some() {
            ctx.request_repaint();
        }
    }
}
