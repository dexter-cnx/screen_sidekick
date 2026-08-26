use sidekick_core::{DEFAULT_JPEG_QUALITY, ExportFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportSettings {
    kind: ExportKind,
    jpeg_quality: u8,
}

impl ExportSettings {
    pub fn new(kind: ExportKind, jpeg_quality: u8) -> Self {
        Self {
            kind,
            jpeg_quality: jpeg_quality.clamp(1, 100),
        }
    }

    pub fn kind(&self) -> ExportKind {
        self.kind
    }

    pub fn set_kind(&mut self, kind: ExportKind) {
        self.kind = kind;
    }

    pub fn jpeg_quality(&self) -> u8 {
        self.jpeg_quality
    }

    pub fn set_jpeg_quality(&mut self, quality: u8) {
        self.jpeg_quality = quality.clamp(1, 100);
    }

    pub fn export_format(&self) -> ExportFormat {
        match self.kind {
            ExportKind::Png => ExportFormat::Png,
            ExportKind::Jpeg => ExportFormat::jpeg(self.jpeg_quality),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self.kind {
            ExportKind::Png => "png",
            ExportKind::Jpeg => "jpg",
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self::new(ExportKind::Jpeg, DEFAULT_JPEG_QUALITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_core_export_policy() {
        let settings = ExportSettings::default();
        assert_eq!(settings.kind(), ExportKind::Jpeg);
        assert_eq!(settings.jpeg_quality(), DEFAULT_JPEG_QUALITY);
        assert_eq!(settings.export_format(), ExportFormat::jpeg(DEFAULT_JPEG_QUALITY));
        assert_eq!(settings.extension(), "jpg");
    }

    #[test]
    fn png_maps_to_png_format_and_extension() {
        let settings = ExportSettings::new(ExportKind::Png, 50);
        assert_eq!(settings.export_format(), ExportFormat::Png);
        assert_eq!(settings.extension(), "png");
    }

    #[test]
    fn jpeg_quality_is_normalized_at_ui_boundary() {
        let mut settings = ExportSettings::new(ExportKind::Jpeg, 0);
        assert_eq!(settings.jpeg_quality(), 1);
        settings.set_jpeg_quality(255);
        assert_eq!(settings.jpeg_quality(), 100);
        assert_eq!(settings.export_format(), ExportFormat::jpeg(100));
    }

    #[test]
    fn changing_kind_preserves_jpeg_quality() {
        let mut settings = ExportSettings::new(ExportKind::Jpeg, 82);
        settings.set_kind(ExportKind::Png);
        assert_eq!(settings.jpeg_quality(), 82);
        settings.set_kind(ExportKind::Jpeg);
        assert_eq!(settings.export_format(), ExportFormat::jpeg(82));
    }
}
