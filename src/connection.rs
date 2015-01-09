use std::io;
use std::thread;

use reader;
use utils::debug;

pub enum ConnEvent {
  Send( String ),
  Recv( String ),
  Abort( String ),
}

pub struct ServerConnection {
  pub host  : String,
  pub port  : u16,
  pub pass  : String,

  tcp   : io::TcpStream,

  chan  : Sender < ConnEvent >,
  listen: Receiver < ConnEvent >,
}

impl ServerConnection {
  pub fn connect( host : &str, port : u16, pass : &str ) 
    -> ServerConnection {
    let target = format!( "{}:{}", host, port );
    let mut tcp = match io::TcpStream::connect( target ) {
      Ok (res)  => res,
      Err (e)   => {
        debug::err( "establishing server connection", e.desc );
      },
    };
    let out = format! ( "connected to {} successfully", target );
    debug::oper( out.as_slice( ) );
    
    let( tx, rx ) = channel( );
    
    if !pass.is_empty( ) {
      tx.send( ConnEvent::Send( format! ( "PASS {}", pass ) ) );
    }
    
    ServerConnection {
      host    : host.to_string( ),
      port    : port,
      pass    : pass.to_string( ),
      tcp     : tcp,
      chan    : tx,
      listen  : rx,
    }
  }

  pub fn close( &mut self ) {
    let out = format! ( "closing connection to {}:{}...", self.host, self.port );
    debug::oper( out.as_slice( ) );
    match self.tcp.close_read( ) {
      Err(e) => debug::err( "closing server read connection", e.desc ),
      _      => debug::info( "read closed successfully" ),
    };
    match self.tcp.close_write( ) {
      Err(e) => debug::err( "closing server write connection", e.desc ),
      _      => debug::info( "write closed successfully" ),
    }
    drop( self.tcp.clone( ) );
    debug::oper( "server connection closed successfully" );
  }
  
  pub fn spawn_reader( &self ) -> thread::Thread {
    thread::Thread::spawn( move || {
      let mut rdr = reader::IrcReader::new(self.tcp.clone(), self.chan.clone());
      rdr.start( );
    } )
  }
}