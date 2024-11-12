use std::error::Error;

pub fn print(error: impl Error) {
    fennec_feedback::error!(target = "fennec", "{}", error);
    fennec_feedback::debug!(target = "fennec", "{:#?}", error);

    if let Some(source) = error.source() {
        fennec_feedback::debug!(target = "fennec", "{:#?}", source);
    }
}

pub fn bail<T>(error: impl Error) -> T {
    print(error);

    // we exit with a non-zero status code to indicate an error
    // if this build is debug build, we will panic instead
    if cfg!(debug_assertions) {
        panic!("bail");
    } else {
        std::process::exit(1);
    }
}
