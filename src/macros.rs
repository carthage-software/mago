#[macro_export]
macro_rules! enum_variants {
    ($e: ty) => {{
        use clap::builder::TypedValueParser;
        use strum::VariantNames;

        clap::builder::PossibleValuesParser::new(<$e>::VARIANTS).map(|s| s.parse::<$e>().unwrap())
    }};
}

#[macro_export]
macro_rules! reporting_arguments {
    () => {};
}
