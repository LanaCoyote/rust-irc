use std::io;
use std::sync::mpsc;
use std::time;

use connection;
use utils::debug;

/// `IrcReader` handles reading from an IRC stream
///
/// # Members
///
/// `tcp` - TcpStream through which IRC is connected
/// `chan` - Send half of the channel used to communicate
pub struct IrcReader {
  tcp   : io::TcpStream,
  chan  : mpsc::Sender < connection::ConnEvent >,
}

impl IrcReader {
  /// `new` creates (but does not start) a new IrcReader struct
  ///
  /// # Arguments
  ///
  /// `tcp` - TcpStream of the IRC client
  /// `tx` - Transmission channel used to talk to the program
  pub fn new ( 
    tcp : io::TcpStream,
    tx : mpsc::Sender < connection::ConnEvent > 
    ) -> IrcReader {
    IrcReader {
      tcp   : tcp,
      chan  : tx,
    }
  }
  
  /// `start` begins reading data from IRC
  ///
  /// # Notes
  ///
  /// - This will block until the connection is closed. Run it in a new thread
  /// so that you can operate on the information you read.
  pub fn start ( &mut self ) {
    let mut try   = 0u8;
    let mut time  = 5;
    let mut read  = io::BufferedReader::new( self.tcp.clone( ) );
    // Check that we're connected to a peer
    match self.tcp.peer_name() {
      Ok ( peer ) => {
        let out = format! ("opening irc reader at {}...", peer );
        debug::oper( out.as_slice( ) );
      },
      Err ( e )   => {
        debug::err( "opening irc reader", e.desc );
        match e.detail {
          Some( det ) => debug::info( det.as_slice( ) ),
          None        => (),
        };
        return;
      },
    }
    
    // Read loop
    loop {
      match read.read_line( ) {
        // Line received, strip the newline and pass it back
        Ok ( line ) => {
          let mut pass = line.clone( );
          pass.pop( );
          match self.chan.send( connection::ConnEvent::Recv( pass ) ) {
            Ok ( _ )  => {
              try  = 0u8;
              time = 5;
            },
            Err ( e ) => {
              debug::err( "irc reader send", "" );
            },
          };
        },
        
        // An error occurred or the connection was severed
        Err ( e )   => {
          match e.kind {
            io::IoErrorKind::EndOfFile => {
              debug::oper( "reader connection closed (eof reported)" );
              break;
            },
            _                          => {
              debug::err( "irc reader", e.desc );
              try += 1u8;
            }
          }
        }
      }
        
      // Fail automatically after 5 tries
      if try > 5 {
        debug::err( "irc reader", "read failed after 5 retries" );
        break;
      // If we encounter an error, try again after a period of time
      } else if try > 0 {
        debug::info( "irc read failed, retrying after some time..." );
        io::timer::sleep( time::Duration::seconds( time ) );
        time *= 2;
      }
    }
    
    debug::oper( "closing irc reader..." );
    match self.chan.send( connection::ConnEvent::Abort( String::from_str( "irc reader closed" ) ) ) {
      Ok ( _ )  => (),
      Err ( e ) => debug::err( "closing irc reader", "" ),
    }
  }
}