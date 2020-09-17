use std::error;

pub fn new(message: &str, explicit_err: Box<dyn error::Error>) -> String {
    let err = format!("{}, err: {}", message, explicit_err);

    return err;
}
