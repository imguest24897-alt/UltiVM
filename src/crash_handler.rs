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
        if let Some(message) = info.payload().downcast_ref::<&str>() {
            eprintln!("Oops! UltiVM has ran into a unknown error and can't show a error window!\r\n{}", message);
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