//! An websocket extension for Routerify.
//!
//! # Examples
//!
//! ```
//! use routerify_websocket;
//!
//! # fn run() {
//! println!("{}", routerify_websocket::add(2, 3));
//! # }
//! # run();
//! ```

pub use self::error::Error;

mod error;

/// This function adds two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
