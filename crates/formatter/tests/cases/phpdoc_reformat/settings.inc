FormatSettings {
    phpdoc_reformat: true,
    phpdoc_align: mago_formatter::settings::PhpdocAlign::Vertical,
    phpdoc_null_position: mago_formatter::settings::PhpdocNullPosition::Last,
    ..mago_formatter::presets::FormatterPreset::Default.settings()
}
