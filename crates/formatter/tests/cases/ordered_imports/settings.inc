FormatSettings {
    sort_uses: true,
    separate_use_types: false,
    expand_use_groups: false,
    use_type_order: UseTypeOrder::try_from(vec![
        UseImportType::Const,
        UseImportType::Class,
        UseImportType::Function,
    ])
    .unwrap(),
    sort_uses_algorithm: UseSortAlgorithm::Alpha,
    sort_uses_case_sensitive: false,
    ..FormatSettings::default()
}
