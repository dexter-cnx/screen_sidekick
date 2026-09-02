use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use sidekick_core::{Annotation, save_annotation_export};

use crate::export_settings::ExportSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRequest {
    output_path: PathBuf,
    settings: ExportSettings,
}

impl ExportRequest {
    pub fn new(output_path: impl Into<PathBuf>, settings: ExportSettings) -> Self {
        Self {
            output_path: output_path.into(),
            settings,
        }
    }

    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    pub fn settings(&self) -> ExportSettings {
        self.settings
    }

    pub fn save(&self, base_path: impl AsRef<Path>, annotations: &[Annotation]) -> Result<(), String> {
        save_annotation_export(
            base_path,
            annotations,
            self.settings.export_format(),
            &self.output_path,
        )
        .map_err(|error| error.to_string())
    }
}

pub fn suggested_export_filename(settings: ExportSettings) -> String {
    suggested_export_filename_at(settings, unix_timestamp_seconds())
}

fn suggested_export_filename_at(settings: ExportSettings, timestamp_seconds: u64) -> String {
    format!("Screen-Sidekick-{timestamp_seconds}.{}", settings.extension())
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use sidekick_core::{DEFAULT_JPEG_QUALITY, ExportFormat};

    use super::*;
    use crate::export_settings::{ExportKind, ExportSettings};

    #[test]
    fn suggested_filename_tracks_selected_format() {
        let jpeg = ExportSettings::new(ExportKind::Jpeg, DEFAULT_JPEG_QUALITY);
        let png = ExportSettings::new(ExportKind::Png, DEFAULT_JPEG_QUALITY);

        assert_eq!(
            suggested_export_filename_at(jpeg, 123),
            "Screen-Sidekick-123.jpg"
        );
        assert_eq!(
            suggested_export_filename_at(png, 123),
            "Screen-Sidekick-123.png"
        );
    }

    #[test]
    fn request_preserves_export_settings() {
        let settings = ExportSettings::new(ExportKind::Jpeg, 82);
        let request = ExportRequest::new("capture.jpg", settings);

        assert_eq!(request.output_path(), Path::new("capture.jpg"));
        assert_eq!(request.settings().export_format(), ExportFormat::jpeg(82));
    }
}
