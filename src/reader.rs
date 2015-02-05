use std::old_io as io;
use std::sync::mpsc::Sender;  // sender channel for passing back data
use std::time::Duration;      // used for the sleep timer

// import custom modules
use connection::ConnEvent;    // used for passing back messages to the client
use utils::debug;             // output debug for logging errors

type   TTRY                    = u8;
type   TTIMEOUT                = i64;

static IRC_TRY_INITIAL  : TTRY = 0;  // initial number of tries attempted
static IRC_TRY_SUCCESS  : TTRY = 0;  // set the try value to this on success
static IRC_TRY_FAILURE  : TTRY = 1;  // try increment on failure
static IRC_TRY_LIMIT    : TTRY = 5;  // maximum number of tries before failing
static IRC_READ_TIMEOUT : TTIMEOUT = 5;  // initial time between irc reads
static IRC_READ_MULT    : TTIMEOUT = 2;  // multiply timeout by this on fail

/// `IrcReader` handles reading from an IRC stream
///
/// # Members
///
/// `tcp` - TcpStream through which IRC is connected
/// `chan` - Send half of the channel used to communicate
pub struct IrcReader {
  tcp   : io::TcpStream,
  chan  : Sender < ConnEvent >,
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
    tx : Sender < ConnEvent > 
    ) -> IrcReader {
    IrcReader {
      tcp   : tcp,
      chan  : tx,
    }
  }
  
  /// `has_peer` returns whether the TCP connection has a valid peer
  ///
  /// # Returns
  ///
  /// true if the connection is valid, false otherwise.
  fn has_peer ( &mut self ) -> bool {
    // get the peer name from our tcp connection
    match self.tcp.peer_name() {
      // we did it, we're connected properly
      Ok ( peer ) => {
        debug::oper( format! ( "irc reader opened at {} successfully", peer ) );
        true
      },
      
      // couldn't connect for some reason
      Err ( e )   => {
        debug::err( "opening irc reader", e.desc );
        match e.detail {
          Some ( det ) => debug::info( det.as_slice( ) ),
          None         => (),
        };
        false
      },
    }
  }
  
  /// `handle_read_success` parses and sends back a line read from IRC
  ///
  /// # Arguments
  ///
  /// * `line` - the line read from the TcpStream
  /// * `try` - the attempt number
  ///
  /// # Returns
  ///
  /// IRC_TRY_SUCCESS if the operation completed successfully. Otherwise it
  /// returns try plus IRC_TRY_FAILURE.
  fn handle_read_success ( &self, line : String, try : TTRY ) -> TTRY {
    // trim whitespace (including the newline)
    let pass = line.as_slice( ).trim_right( );
    
    // pass our message back to the client
    match self.chan.send( ConnEvent::Recv( pass.to_string( ) ) ) {
      // it worked, reset the try counter
      Ok ( _ )  => {
        if try > IRC_TRY_SUCCESS {
          debug::info( format! ( "successful read after {} attempts", try ) );
        }
        IRC_TRY_SUCCESS
      },
      
      // an error occurred
      Err ( _ ) => {
        debug::err( "irc reader send", "receiver hung up" );
        IRC_TRY_LIMIT
      },
    }
  }
  
  /// `handle_read_failure` reports an error that occurred while reading
  ///
  /// # Arguments
  ///
  /// * `e` - the error returned by the TcpStream
  /// * `try` - the attempt number
  ///
  /// # Returns
  ///
  /// IRC_TRY_LIMIT if the error is an EOF. Otherwise it increments try by
  /// IRC_TRY_FAILURE.
  fn handle_read_failure ( &self, e : io::IoError, try : TTRY ) -> TTRY {
    match e.kind {
      // eof means the tcp connection was closed
      io::IoErrorKind::EndOfFile => {
        debug::err( "irc reader receive", "eof reported, closing connection" );
        IRC_TRY_LIMIT
      },
      
      // all other errors
      _                          => {
        debug::err( "irc reader receive", e.desc );
        match e.detail {
          Some ( det ) => debug::info( det.as_slice( ) ),
          None         => (),
        };
        try + IRC_TRY_FAILURE
      },
    }
  }
  
  /// `get_next_try` gets whether the loop should continue
  ///
  /// # Arguments
  ///
  /// * `try` - the attempt number
  /// * `time` - the timeout between attempts
  ///
  /// # Returns
  ///
  /// None if the reader has attempted too many reads without success. If the
  /// read failed, it sleeps and returns the timeout times IRC_READ_MULT. If the
  /// read succeeded it returns IRC_READ_TIMEOUT.
  fn get_next_try ( &self, try : TTRY, time : TTIMEOUT ) -> Option < TTIMEOUT > {
    // end the reader if we've gone over the try limit
    if try >= IRC_TRY_LIMIT {
      debug::err( "irc reader", format! ( "failed after {} retries", try ) );
      None
      
    // if we've failed, don't retry immediately
    } else if try > IRC_TRY_SUCCESS {
      debug::info( format! ( "retrying after {} seconds...", time ) );
      io::timer::sleep( Duration::seconds( time ) );
      Some( time * IRC_READ_MULT )
      
    // we did it, reset the timer
    } else {
      Some( IRC_READ_TIMEOUT )
    }
  }
  
  /// `start` begins reading data from IRC
  ///
  /// # Notes
  ///
  /// - This will block until the connection is closed. Run it in a new thread
  /// so that you can operate on the information you read.
  pub fn start ( &mut self ) {
    let mut try   = IRC_TRY_INITIAL;
    let mut time  = IRC_READ_TIMEOUT;
    let mut read  = io::BufferedReader::new( self.tcp.clone( ) );
    // Check that we're connected to a peer
    if !self.has_peer( ) {
      return;
    }
    
    // Read loop
    loop {
      try = match read.read_line( ) {
        Ok ( line ) => self.handle_read_success( line, try ),
        Err ( e )   => self.handle_read_failure( e, try ),
      };
        
      // Fail automatically after 5 tries
      match self.get_next_try( try, time ) {
        None       => break,
        Some ( t ) => time = t,
      };
    }
    
    debug::oper( "closing irc reader..." );
    match self.chan.send( ConnEvent::Abort( String::from_str( "irc reader closed" ) ) ) {
      Ok ( _ )  => (),
      Err ( _ ) => debug::err( "closing irc reader", "" ),
    }
  }
}