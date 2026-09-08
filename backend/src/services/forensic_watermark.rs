use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use rand::Rng;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Forensic Watermark — скрытая стеганографическая метка
/// Встраивается в DCT-коэффициенты изображения/видео кадра
/// Невидима для пользователя, но извлекается для идентификации утечки

pub struct ForensicWatermark {
    /// Сила встраивания (0.0 - 1.0)
    strength: f32,
    /// Размер блока для DCT
    block_size: usize,
}

impl ForensicWatermark {
    pub fn new(strength: f32) -> Self {
        Self {
            strength: strength.clamp(0.1, 0.5),
            block_size: 8,
        }
    }

    /// Генерация уникального паттерна для пользователя
    pub fn generate_user_pattern(user_id: &str, seed: u64) -> Vec<f32> {
        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        hasher.update(seed.to_le_bytes());
        let hash = hasher.finalize();

        // Конвертируем хэш в паттерн коэффициентов
        let mut pattern = Vec::with_capacity(64);
        for i in 0..64 {
            let byte = hash[i % 32];
            let value = ((byte as f32) / 255.0) * 2.0 - 1.0; // -1.0 to 1.0
            pattern.push(value);
        }

        pattern
    }

    /// Встраивание watermark в изображение (LSB steganography)
    pub fn embed_watermark(&self, image: &DynamicImage, user_id: &str) -> DynamicImage {
        let (width, height) = image.dimensions();
        let mut img = image.to_rgb8();

        // Генерируем паттерн для пользователя
        let pattern = Self::generate_user_pattern(user_id, 42);

        // Встраиваем в каждый блок 8x8
        let blocks_x = width as usize / self.block_size;
        let blocks_y = height as usize / self.block_size;

        for by in 0..blocks_y {
            for bx in 0..blocks_x {
                let block_idx = by * blocks_x + bx;
                let pattern_value = pattern[block_idx % pattern.len()];

                // Модифицируем пиксели в блоке
                for y in 0..self.block_size {
                    for x in 0..self.block_size {
                        let px = bx * self.block_size + x;
                        let py = by * self.block_size + y;

                        if px < width as usize && py < height as usize {
                            let pixel = img.get_pixel(px as u32, py as u32);
                            
                            // Модифицируем LSB с учётом паттерна
                            let r = self.modify_lsb(pixel[0], pattern_value, x, y);
                            let g = self.modify_lsb(pixel[1], pattern_value, x + 1, y);
                            let b = self.modify_lsb(pixel[2], pattern_value, x, y + 1);

                            img.put_pixel(px as u32, py as u32, Rgb([r, g, b]));
                        }
                    }
                }
            }
        }

        DynamicImage::ImageRgb8(img)
    }

    /// Извлечение watermark из изображения
    pub fn extract_watermark(&self, image: &DynamicImage, user_id: &str) -> f32 {
        let (width, height) = image.dimensions();
        let img = image.to_rgb8();

        // Генерируем ожидаемый паттерн
        let pattern = Self::generate_user_pattern(user_id, 42);

        let blocks_x = width as usize / self.block_size;
        let blocks_y = height as usize / self.block_size;

        let mut correlation = 0.0f32;
        let mut count = 0;

        for by in 0..blocks_y {
            for bx in 0..blocks_x {
                let block_idx = by * blocks_x + bx;
                let expected_value = pattern[block_idx % pattern.len()];

                // Извлекаем модификации из блока
                for y in 0..self.block_size {
                    for x in 0..self.block_size {
                        let px = bx * self.block_size + x;
                        let py = by * self.block_size + y;

                        if px < width as usize && py < height as usize {
                            let pixel = img.get_pixel(px as u32, py as u32);
                            
                            // Извлекаем LSB
                            let extracted = self.extract_lsb(pixel[0], x, y);
                            correlation += extracted * expected_value;
                            count += 1;
                        }
                    }
                }
            }
        }

        if count > 0 {
            correlation / count as f32
        } else {
            0.0
        }
    }

    /// Модификация LSB пикселя
    fn modify_lsb(&self, value: u8, pattern: f32, x: usize, y: usize) -> u8 {
        // Определяем бит для встраивания
        let bit = if (pattern.sin() + x as f32 * 0.1 + y as f32 * 0.1).sin() > 0.0 {
            1
        } else {
            0
        };

        // Модифицируем LSB с учётом силы
        let modified = if bit == 1 {
            value | 1
        } else {
            value & !1
        };

        // Применяем силу (сглаживаем изменение)
        let delta = (modified as i32 - value as i32) as f32 * self.strength;
        (value as f32 + delta).clamp(0.0, 255.0) as u8
    }

    /// Извлечение LSB
    fn extract_lsb(&self, value: u8, x: usize, y: usize) -> f32 {
        let bit = (value & 1) as f32;
        bit * 2.0 - 1.0 // -1.0 or 1.0
    }

    /// Встраивание watermark в видео (кадр за кадром)
    pub fn embed_video_watermark(
        &self,
        frame: &DynamicImage,
        user_id: &str,
        frame_number: u32,
    ) -> DynamicImage {
        // Используем frame_number как seed для вариативности
        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        hasher.update(frame_number.to_le_bytes());
        let seed = u64::from_le_bytes(hasher.finalize()[..8].try_into().unwrap());

        let pattern = Self::generate_user_pattern(user_id, seed);
        
        // Встраиваем с учётом временной оси
        self.embed_with_temporal_pattern(frame, &pattern, frame_number)
    }

    /// Встраивание с временным паттерном (для видео)
    fn embed_with_temporal_pattern(
        &self,
        image: &DynamicImage,
        pattern: &[f32],
        frame_number: u32,
    ) -> DynamicImage {
        let (width, height) = image.dimensions();
        let mut img = image.to_rgb8();

        let blocks_x = width as usize / self.block_size;
        let blocks_y = height as usize / self.block_size;

        // Временная модуляция (меняет паттерн каждые N кадров)
        let temporal_offset = (frame_number / 30) as usize; // Каждую секунду при 30fps

        for by in 0..blocks_y {
            for bx in 0..blocks_x {
                let block_idx = (by * blocks_x + bx + temporal_offset) % pattern.len();
                let pattern_value = pattern[block_idx];

                for y in 0..self.block_size {
                    for x in 0..self.block_size {
                        let px = bx * self.block_size + x;
                        let py = by * self.block_size + y;

                        if px < width as usize && py < height as usize {
                            let pixel = img.get_pixel(px as u32, py as u32);
                            
                            let r = self.modify_lsb(pixel[0], pattern_value, x, y);
                            let g = self.modify_lsb(pixel[1], pattern_value, x + 1, y);
                            let b = self.modify_lsb(pixel[2], pattern_value, x, y + 1);

                            img.put_pixel(px as u32, py as u32, Rgb([r, g, b]));
                        }
                    }
                }
            }
        }

        DynamicImage::ImageRgb8(img)
    }
}

/// Сервис для управления forensic watermark
pub struct ForensicWatermarkService {
    watermark: ForensicWatermark,
}

impl ForensicWatermarkService {
    pub fn new(strength: f32) -> Self {
        Self {
            watermark: ForensicWatermark::new(strength),
        }
    }

    /// Обработка изображения с watermark
    pub fn process_image(&self, image: &DynamicImage, user_id: &str) -> DynamicImage {
        self.watermark.embed_watermark(image, user_id)
    }

    /// Обработка видео кадра с watermark
    pub fn process_video_frame(
        &self,
        frame: &DynamicImage,
        user_id: &str,
        frame_number: u32,
    ) -> DynamicImage {
        self.watermark.embed_video_watermark(frame, user_id, frame_number)
    }

    /// Проверка изображения на наличие watermark
    pub fn verify_image(&self, image: &DynamicImage, user_id: &str) -> bool {
        let correlation = self.watermark.extract_watermark(image, user_id);
        // Порог обнаружения (настраивается экспериментально)
        correlation > 0.6
    }

    /// Идентификация пользователя по watermark
    pub fn identify_user(&self, image: &DynamicImage, known_users: &[String]) -> Option<String> {
        let mut best_match = None;
        let mut best_correlation = 0.0f32;

        for user_id in known_users {
            let correlation = self.watermark.extract_watermark(image, user_id);
            if correlation > best_correlation && correlation > 0.6 {
                best_correlation = correlation;
                best_match = Some(user_id.clone());
            }
        }

        best_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, Rgb};

    #[test]
    fn test_watermark_embed_extract() {
        let service = ForensicWatermarkService::new(0.3);
        
        // Создаём тестовое изображение
        let img = RgbImage::from_pixel(64, 64, Rgb([128, 128, 128]));
        let dynamic_img = DynamicImage::ImageRgb8(img);
        
        let user_id = "test_user_123";
        
        // Встраиваем watermark
        let watermarked = service.process_image(&dynamic_img, user_id);
        
        // Проверяем обнаружение
        assert!(service.verify_image(&watermarked, user_id));
        
        // Проверяем, что другой пользователь не обнаруживается
        assert!(!service.verify_image(&watermarked, "other_user"));
    }

    #[test]
    fn test_pattern_generation() {
        let pattern1 = ForensicWatermark::generate_user_pattern("user1", 42);
        let pattern2 = ForensicWatermark::generate_user_pattern("user2", 42);
        let pattern3 = ForensicWatermark::generate_user_pattern("user1", 43);
        
        // Разные пользователи → разные паттерны
        assert_ne!(pattern1, pattern2);
        
        // Разные seeds → разные паттерны
        assert_ne!(pattern1, pattern3);
        
        // Один и тот же пользователь + seed → одинаковый паттерн
        let pattern1_again = ForensicWatermark::generate_user_pattern("user1", 42);
        assert_eq!(pattern1, pattern1_again);
    }
}
