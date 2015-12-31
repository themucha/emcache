// Declare sub modules
pub mod buffered_stream;
pub mod errors;
pub mod stats;
pub mod transport;
pub mod typedefs;

// internal stuff
mod tests;  // needed to be part of the compilation unit in test mode


// Export our public api
pub use self::buffered_stream::BufferedStream;
pub use self::errors::TcpTransportError;
pub use self::stats::TransportStats;
pub use self::transport::TcpTransport;
pub use self::typedefs::TcpTransportResult;
