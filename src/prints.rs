#[macro_export]
macro_rules! console_answer {
    ($($arg:tt)*) => {
        print!("[🤖] ");
        println!($($arg)*);
    };
}
