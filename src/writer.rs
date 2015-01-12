use std::io;

/// `IrcWriter` is a typedef of a buffered writer to the IRC stream
pub type IrcWriter = io::LineBufferedWriter < io::TcpStream >;