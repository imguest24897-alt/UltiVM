use std::panic;

pub fn setup_crash_handler() {
    panic::set_hook(Box::new(|info| {
        eprintln!("Application crashed!");
        if let Some(location) = info.location() {
            eprintln!(
                "Crash occurred at file '{}' line {}",
                location.file(),
                location.line()
            );
        }
        match info.payload().downcast_ref::<&str>() {
            Some(message) => {
                eprintln!("Oops! UltiVM has run into an unknown error and can't show an error window!\r\n{}", message);
            }
            None => {
                eprintln!("Oops! UltiVM has run into an unknown error and can't show an error window!");
            }
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_crash_handler() {
        setup_crash_handler();
        panic!("This is a test panic!");
    }
}