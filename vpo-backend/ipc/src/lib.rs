pub mod communication_constants;
pub mod error;
pub mod ipc_message;
pub mod ipc_server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
