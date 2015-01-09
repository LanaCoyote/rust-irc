mod connection;
mod event;
mod info;
mod reader;
mod utils;

pub struct Client <'cl> {
  info    : info::IrcInfo,
  conn    : connection::ServerConnection,
  events  : event::EventDispatcher,
}

impl <'cl> Client <'cl> {
  pub fn new <'a> ( 
    host : &str, 
    port : u16, 
    pass : &str, 
    info : info::IrcInfo <'a> 
  ) -> Client <'a>
  {
    Client {
      info    : info,
      conn    : connection::ServerConnection::connect( host, port, pass ),
      events  : event::EventDispatcher,
    }
  }
  
  pub fn disconnect ( &self ) {
    
  }
}