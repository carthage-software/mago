use fennec_feedback::create_progress_bar;
use fennec_feedback::remove_progress_bar;
use fennec_feedback::ProgressBarTheme;
use fennec_formatter::format;
use fennec_interner::ThreadedInterner;
use fennec_parser::parse_source;
use fennec_source::error::SourceError;
use fennec_source::SourceIdentifier;
use fennec_source::SourceManager;

use crate::formatter::config::FormatterConfiguration;

pub mod config;

#[derive(Debug)]
pub struct FormatterService {
    configuration: FormatterConfiguration,
    interner: ThreadedInterner,
    source_manager: SourceManager,
}

impl FormatterService {
    pub fn new(
        configuration: FormatterConfiguration,
        interner: ThreadedInterner,
        source_manager: SourceManager,
    ) -> Self {
        Self { configuration, interner, source_manager }
    }

    /// Runs the formatting process.
    pub async fn run(&self) -> Result<usize, SourceError> {
        // Process sources concurrently
        self.process_sources(self.source_manager.user_defined_source_ids().collect()).await
    }

    #[inline]
    async fn process_sources<'a>(&self, source_ids: Vec<SourceIdentifier>) -> Result<usize, SourceError> {
        let settings = self.configuration.get_settings();
        let mut handles = Vec::with_capacity(source_ids.len());

        let source_pb = create_progress_bar(source_ids.len(), "📂  Loading", ProgressBarTheme::Red);
        let parse_pb = create_progress_bar(source_ids.len(), "🧩  Parsing", ProgressBarTheme::Blue);
        let format_pb = create_progress_bar(source_ids.len(), "✨  Formatting", ProgressBarTheme::Magenta);
        let write_pb = create_progress_bar(source_ids.len(), "🖊️  Writing", ProgressBarTheme::Green);

        for source_id in source_ids.into_iter() {
            handles.push(tokio::spawn({
                let interner = self.interner.clone();
                let manager = self.source_manager.clone();
                let source_pb = source_pb.clone();
                let parse_pb = parse_pb.clone();
                let format_pb = format_pb.clone();
                let write_pb = write_pb.clone();

                async move {
                    // Step 1: load the source
                    let source = manager.load(source_id)?;
                    source_pb.inc(1);

                    fennec_feedback::debug!("> parsing program: {}", interner.lookup(&source.identifier.0));

                    // Step 2: parse the source
                    let (program, error) = parse_source(&interner, &source);
                    parse_pb.inc(1);

                    if let Some(error) = error {
                        let source_name = interner.lookup(&source.identifier.0);
                        fennec_feedback::error!("skipping formatting for source '{}', {} ", source_name, error);

                        format_pb.inc(1);
                        write_pb.inc(1);

                        return Result::<_, SourceError>::Ok(());
                    }

                    fennec_feedback::debug!("> formatting program: {}", interner.lookup(&program.source.0));

                    // Step 3: format the source
                    let formatted = format(settings, &interner, &source, &program);
                    format_pb.inc(1);

                    fennec_feedback::debug!("> writing program: {}", interner.lookup(&program.source.0));

                    // Step 4: write the formatted source
                    manager.write(source.identifier, formatted)?;
                    write_pb.inc(1);

                    fennec_feedback::debug!("< formatted program: {}", interner.lookup(&program.source.0));

                    Result::<_, SourceError>::Ok(())
                }
            }));
        }

        let mut count = 0;
        for handle in handles {
            handle.await.expect("failed to format files, this should never happen.")?;

            count += 1;
        }

        remove_progress_bar(source_pb);
        remove_progress_bar(parse_pb);
        remove_progress_bar(format_pb);
        remove_progress_bar(write_pb);

        Ok(count)
    }
}
