use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Raw socket error: {0}")]
    Rawsock(#[from] rawsock::Error),

    #[error("I/O: {0}")]
    IO(#[from] std::io::Error),

    #[error("Win32 error code {0} (0x{0:x})")]
    Win32(u32),

    #[error("ersatz could not find the default IP route")]
    DefaultRouteMissing,

    #[error("ersatz could not find the default netwrok interface")]
    DefaultInterfaceMissing,

    #[error("ersatz could not identify the default network interface")]
    DefaultInterfaceUnidentified,

    #[error("ersatz could not determine the IP address of the default netwrok interface")]
    DefaultInterfaceNoIPAddr,

    #[error("ersatz could not determine the MAC address of the default netwrok interface")]
    DefaultInterfaceNoMACAddr,
}
