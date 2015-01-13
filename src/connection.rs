use std::io;
use std::sync::mpsc;

use utils::debug;

/// `ConnEvent` defines the various actions our connection might use
///
/// # Options
///
/// `Send` - send this message to the server
/// `Recv` - this message was received from the server
/// `Abort` - shut down connection and close
pub enum ConnEvent {
  Send( String ),
  Recv( String ),
  Abort( String ),
}

/// `ServerConnection` manages an IRC connection
///
/// # Members
///
/// `host` - the host of the server we're connected to
/// `port` - the port we're connected to the server through
/// `pass` - the password of the server
/// `tcp` - the TcpStream to the server
/// `chan` - transmission half of our thread channel
/// `listen` - listener half of our thread channel
pub struct ServerConnection {
  pub host  : String,
  pub port  : u16,
  pub pass  : String,

  pub tcp   : io::TcpStream,

  pub chan  : mpsc::Sender < ConnEvent >,
  pub listen: Option < mpsc::Receiver < ConnEvent > >,
}

impl ServerConnection {
  /// `connect` establishes a new connection to a given host and port
  ///
  /// # Arguments
  ///
  /// `host` - server host to connect to
  /// `port` - port number to connect to the server on
  /// `pass` - password for the irc server
  ///
  /// # Returns
  ///
  /// A new ServerConnection struct that is connected to the target server
  pub fn connect ( host : &str, port : u16, pass : &str ) 
    -> ServerConnection {
    // Format the server address and attempt a connection
    let target = format!( "{}:{}", host, port );
    let out = format!( "establishing connection to {}...", target );
    debug::oper( out.as_slice( ) );
    let tcp = match io::TcpStream::connect( target.as_slice( ) ) {
      Ok (res)  => res,
      Err (e)   => {
        debug::err( "establishing server connection", e.desc );
        panic! ( "connection failure is not implemented" );
      },
    };
    debug::oper( "connection established!" );
    
    // Create a channel for communication between spawned threads
    let( tx, rx ) = mpsc::channel( );
    
    // Send a password message to the message buffer
    if !pass.is_empty( ) {
      match tx.send( ConnEvent::Send( format! ( "PASS {}", pass ) ) ) {
        Ok ( _ )  => (),
        Err ( e ) => debug::err( "sending password to server", "" ),
      }
    }
    
    // Build the server struct
    ServerConnection {
      host    : host.to_string( ),
      port    : port,
      pass    : pass.to_string( ),
      tcp     : tcp,
      chan    : tx,
      listen  : Some( rx ),
    }
  }

  /// `close` severs the connection with the server and shuts down the stream
  pub fn close( &mut self ) {
    let out = format! ( "closing connection to {}:{}...", self.host, self.port );
    debug::oper( out.as_slice( ) );
    
    // Close the read stream
    match self.tcp.close_read( ) {
      Err(e) => debug::err( "closing server read connection", e.desc ),
      _      => debug::info( "read closed successfully" ),
    };
    
    // Close the write stream
    match self.tcp.close_write( ) {
      Err(e) => debug::err( "closing server write connection", e.desc ),
      _      => debug::info( "write closed successfully" ),
    }
    
    // Now close ourselves
    drop( self.tcp.clone( ) );
    debug::oper( "server connection closed successfully" );
  }
  
  /// `spin_reader` spins up a new IrcReader in a separate thread
  // pub fn spin_reader ( &self ) {
    // let rthread = thread::Thread::spawn( move || {
      // let mut rdr = reader::IrcReader::new(self.tcp.clone(), self.chan.clone());
      // rdr.start( );
    // } );
  // }
  
  /// `spin_writer` spins up a new IrcWriter and returns a handle to it
  pub fn spin_writer( &self ) -> io::LineBufferedWriter < io::TcpStream > {
    io::LineBufferedWriter::new( self.tcp.clone() )
  }
}