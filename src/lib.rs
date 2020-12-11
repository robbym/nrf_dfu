pub mod archive;
pub mod dfu;
pub mod protocol;
mod slip;
pub mod updater;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
