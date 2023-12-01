use cfg_if::cfg_if;

pub mod communication_constants;
pub mod error;
pub mod ipc_message;
#[cfg(any(windows, unix))]
pub mod send_buffer;

cfg_if! {
    if #[cfg(any(windows, unix))] {
        pub mod ipc_server;
        pub mod file_server;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
