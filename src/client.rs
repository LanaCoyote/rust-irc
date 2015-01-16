// import built in modules
use std::io;
use std::sync;
use std::sync::mpsc;
use std::thread;

// import custom modules
use connection;
use ctcp;
use info;
use message;
use reader;
use utils::debug;

/// `Client` manages IRC connection and communication.
///
/// # Fields
///
/// * `info` - IrcInfo struct that contains information such as the client
/// nick, username, and channels to connect to on startup.
/// * `conn` - ServerConnection struct that maintains the client's connection
/// to the server.
/// * `writer` - Buffered writer that controls writing to the TcpStream
pub struct Client {
  pub info    : Box < info::IrcInfo >,
  pub conn    : connection::ServerConnection,
  pub writer  : io::LineBufferedWriter < io::TcpStream >,
  
  thread      : Option < thread::Thread >,
}

impl Client {
  /// `connect` connects to an IRC server with the given info.
  ///
  /// # Arguments
  ///
  /// * `host` - host of the server to connect to
  /// * `port` - port to transmit to the server over
  /// * `pass` - password of the server. Use a blank string if the server has
  /// no password
  /// * `info` - IrcInfo struct that contains the info to use on the client
  pub fn connect ( 
    host : &str, 
    port : u16, 
    pass : &str, 
    info : Box < info::IrcInfo >
  ) -> Client
  {
    let conn : connection::ServerConnection = 
      connection::ServerConnection::connect( host, port, pass );
    let wrt = conn.spin_writer( );
    Client {
      info        : info,
      conn        : conn,
      writer      : wrt,
      thread      : None,
    }
  }
  
  /// `close` shuts down the IRC client and frees up memory.
  fn close( &mut self ) {
    self.conn.close( );
  }
  
  /// `callback_ping` is called whenever a ping message is received from the
  /// server.
  ///
  /// # Arguments
  ///
  /// * `w` - mutable reference to the TcpStream writer
  /// * `msg` - original ping message
  fn callback_ping( 
    w : &mut io::LineBufferedWriter < io::TcpStream >,
    msg : message::Message
  ) {
    debug::info( "responding to ping request from server..." );
    
    // invert the message and send it back to the server
    match w.write_line( msg.pong( ).raw.as_slice( ) ) {
      Ok ( _ )  => (),
      Err ( e ) => debug::err( "ping response", e.desc ),
    };
  }
  
  /// `callback_notice` is called whenever a notice message is received
  ///
  /// # Arguments
  ///
  /// * `w` - mutable reference to the TcpStream writer
  /// * `i` - reference to the client info
  /// * `registered` - reference to the boolean that determines if we're regged
  fn callback_notice( 
    w : &mut io::LineBufferedWriter < io::TcpStream >,
    i : Box < info::IrcInfo >,
    registered : &mut bool
  ) {
    if !*registered {
      // assemble our messages to the server
      debug::info( "registering on server..." );
      let nickline = format! ( "NICK {}", i.nick_name );
      let userline = format! ( "USER {} * * :{}", 
        i.user_name, i.real_name );
      
      // send them (order is important)
      match w.write_line( nickline.as_slice() ) {
        Ok ( _ )  => debug::info( "registering nick on server" ),
        Err ( e ) => debug::err( "nick registration", e.desc ),
      };
      match w.write_line( userline.as_slice() ) {
        Ok ( _ )  => debug::info( "registering username on server" ),
        Err ( e ) => debug::err( "username registration", e.desc ),
      };
      
      // mark ourselves as registered
      *registered = true;
    }
  }
  
  /// `callback_welcome` is called whenever a welcome code (001) is received
  ///
  /// # Arguments
  ///
  /// * `w` - mutable reference to the TcpStream writer
  /// * `i` - reference to the client info
  fn callback_welcome(
    w : &mut io::LineBufferedWriter < io::TcpStream >,
    i : Box < info::IrcInfo >
  ) {
    debug::info( "joining channels..." );
    for chan in i.channels.iter() {
      let joinline  = format! ( "JOIN {}", chan );
      let debugline = format! ( "joining channel {}", chan );
      match w.write_line( joinline.as_slice( ) ) {
        Ok ( _ )  => debug::info( debugline.as_slice( ) ),
        Err ( e ) => debug::err( debugline.as_slice( ), e.desc ),
      };
    }
  }
  
  fn callback_names( mut i : Box < info::IrcInfo >, msg : message::Message ) {
    debug::info( "getting name list..." );
    i.prep_channel_names( msg );
  }
  
  fn callback_end_of_names( mut i : Box < info::IrcInfo >, msg : message::Message ) {
    debug::info( "got name list ok!" );
    i.set_channel_names( msg.param( 2 ).unwrap( ).to_string( ) );
  }
  
  /// `handle_recv` is called whenever a Recv ConnEvent is read
  ///
  /// # Arguments
  ///
  /// * `s` - String contents of the ConnEvent, the message received
  /// * `w` - mutable reference to the TcpStream writer
  /// * `i` - reference to the client info
  /// * `registered` - ref to boolean that determines if we're regged on the server
  /// * `chan` - channel to send back our final message on
  fn handle_recv( 
    s : String,                                        // raw message received
    w : &mut io::LineBufferedWriter < io::TcpStream >, // writer to output to
    mut i : Box < info::IrcInfo >,               // irc client info
    registered : &mut bool,                            // are we registered?
    chan : &mut mpsc::Sender < message::Message >      // channel to send msg on
  ) {
    // parse our raw string into a usable message
    let msg = match message::Message::parse( 
      ctcp::ctcp_dequote( ctcp::low_level_dequote( s.clone( ) ) ).as_slice( ) ) {
      Some ( m ) => m,
      None       => {
        debug::err( "parsing IRC message", "message is not an IRC message" );
        debug::info( s.as_slice( ) );
        return;
      },
    };
    
    // update client info if necessary
    i.update_info( msg.clone( ) );
    
    // perform basic callbacks
    match msg.code.as_slice( ) {
      "PING"    => Client::callback_ping( w, msg.clone( ) ),
      "NOTICE"  => Client::callback_notice( w, i, registered ),
      "001"     => Client::callback_welcome( w, i ),
      "353"     => Client::callback_names( i, msg.clone( ) ),
      "366"     => Client::callback_end_of_names( i, msg.clone( ) ),
      _         => (),
    };
    
    // send the message back along our channel
    match chan.send( msg ) {
      Ok ( _ )  => (),
      Err ( _ ) => debug::err( "returning message to user", "" ),
    }
  }
  
  /// `handle_send` is called whenever a Send ConnEvent is read
  ///
  /// # Arguments
  ///
  /// * `s` - String contents of the ConnEvent, the message to send
  /// * `w` - mutable reference to the TcpStream writer
  fn handle_send( s : String, w : &mut io::LineBufferedWriter < io::TcpStream > ) {
    match w.write_line( ctcp::low_level_quote( 
      ctcp::ctcp_quote( s.clone( ) ) ).as_slice( ) ) {
      Ok ( _ )  => (),
      Err ( e ) => debug::err( "writing sent message", e.desc ),
    }
    debug::disp( s.as_slice( ), false );
  }
  
  /// `start_handler` runs the message handling interface
  ///
  /// # Arguments
  ///
  /// * `w` - mutable reference to the TcpStream writer
  /// * `i` - reference to client info
  /// * `chan` - channel to send back completed messages on
  /// * `port` - port to receive incoming events on
  fn start_handler( 
    mut w : io::LineBufferedWriter < io::TcpStream >, // writer to send messages to
    mut i : Box < info::IrcInfo >, // client info
    mut chan : mpsc::Sender < message::Message >,     // channel to send received messages over
    port : mpsc::Receiver < connection::ConnEvent >   // port to receive data on
  ) {
    debug::oper( "starting message handler..." );
    let mut registered  = false;
    let mut realinfo    = i;
    loop {
      match port.recv( ) {
        Ok ( t )  => match t {
          connection::ConnEvent::Send( s ) => Client::handle_send( s, &mut w ),
          connection::ConnEvent::Recv( s ) => Client::handle_recv( s, &mut w, realinfo.clone( ), &mut registered, &mut chan ),
          connection::ConnEvent::Abort( s ) => {
            let stopline = format! ( "client handler aborted: {}", s );
            debug::oper( stopline.as_slice( ) );
            match w.write_line( "QUIT" ) {
              Ok( _ )  => (),
              Err( e ) => debug::err( "sending quit line", e.desc ),
            };
            break;
          },
        },
        Err ( _ ) => {
          debug::err( "client handler", "receive channel closed" );
          break;
        },
      }
    }
    debug::oper( "closing message handler..." );
  }
  
  /// `start_reader` spins up a new reader thread and starts it
  ///
  /// # Arguments
  ///
  /// `tcp` - the TcpStream to read from
  /// `chan` - the channel to send back messages on
  fn start_reader( tcp : io::TcpStream, chan : mpsc::Sender < connection::ConnEvent > ) {
    debug::oper( "starting irc reader thread..." );
    let rthread = thread::Thread::spawn( move || {
      let mut rdr = reader::IrcReader::new( tcp, chan );
      rdr.start( );
    } );
    debug::oper( "irc reader started successfully" );
  }
  
  /// `send_msg` sends a Message struct to the IRC server.
  ///
  /// # Arguments
  ///
  /// `m` - Message struct to transmit.
  pub fn send_msg( &mut self, m : message::Message ) {
    match self.conn.chan.send( connection::ConnEvent::Send( m.raw ) ) {
      Ok ( _ )  => (),
      Err ( _ ) => debug::err( "sending message to client", "" ),
    }
  }
  
  /// `send_str` sends a raw string to the IRC server.
  ///
  /// # Arguments
  ///
  /// `s` - string slice to transmit.
  pub fn send_str( &mut self, s : &str ) {
    match self.conn.chan.send( connection::ConnEvent::Send( s.to_string( ) ) ) {
      Ok ( _ )  => (),
      Err ( _ ) => debug::err( "sending raw message to client", "" ),
    }
  }

  /// `start_thread` spins up a reader and message handler on a new thread and
  /// manages IRC communication asynchronously.
  ///
  /// # Returns
  ///
  /// A tuple containing:
  /// * Receiver the client will send parsed IRC messages to
  /// * A "cooked" version of the client
  pub fn start_thread ( mut self ) -> ( mpsc::Receiver < message::Message >, Client )  {
    match self.thread {
      Some ( _ )  => {
        debug::err( "starting client thread", "client thread already started" );
        let (_,fakerx) = mpsc::channel( );
        ( fakerx, self )
      },
      None        => {
        debug::oper( "starting client thread..." );
        let (tx,rx) = mpsc::channel( );
        let params  = ( self.conn.tcp.clone( ), self.conn.chan.clone( ), 
          self.conn.spin_writer( ), self.info.clone( ), 
          self.conn.listen.expect( "no receiver found" ) );
        self.thread = Some( thread::Thread::spawn( move || {
          Client::start_reader( params.0, params.1 );
          Client::start_handler( params.2, params.3, tx.clone( ), params.4 );
        } ) );
        self.conn.listen = None;
        ( rx, self )
      },
    }
  }
  
  /// `stop` ends the client thread.
  pub fn stop( &mut self ) {
    match self.conn.chan.send( connection::ConnEvent::Abort( 
      String::from_str( "stop called from client" ) ) ) {
      Ok ( _ )  => self.close( ),
      Err ( _ ) => debug::err( "stopping client", "" ),
    }
  }
  
  // Abstraction methods
  
  /// `send_ctcp` sends a CTCP tagged message to the target
  ///
  /// # Arguments
  ///
  /// * `target` - target client of the message
  /// * `message` - ctcp message to send, command and parameters
  pub fn send_ctcp( &mut self, target : &str, message : &str ) {
    self.message( target, ctcp::tag( message ).as_slice( ) );
  }
  
  /// `send_ctcp_reply` sends a response to a CTCP message
  ///
  /// # Arguments
  ///
  /// * `target` - target client of the message
  /// * `message` - ctcp message to send, command and parameters
  ///
  /// # Notes
  ///
  /// * Unlike `send_ctcp`, `send_ctcp_reply` is sent as a NOTICE, as specified
  /// in the CTCP documentation.
  pub fn send_ctcp_reply( &mut self, target : &str, message : &str ) {
    self.notice( target, ctcp::tag( message ).as_slice( ) );
  }
  
  /// `identify` identifies with the NickServ service
  ///
  /// # Arguments
  ///
  /// * `password` - NickServ password to identify with
  pub fn identify( &mut self, password : &str ) {
    let sendline = format! ( "IDENTIFY {}", password );
    self.message( "NickServ", sendline.as_slice( ) );
  }
  
  /// `message` sends a private message to the target
  ///
  /// # Arguments
  ///
  /// * `target` - target of the message
  /// * `message` - body of the message
  pub fn message( &mut self, target : &str, message : &str ) {
    let sendline = format! ( "PRIVMSG {} :{}", target, message );
    self.send_str( sendline.as_slice( ) );
  }
  
  /// `notice` sends a notice message to the target
  ///
  /// # Arguments
  ///
  /// * `target` - target of the message
  /// * `message` - body of the message
  ///
  /// # Notes
  ///
  /// * NOTICE is different from PRIVMSG because a NOTICE never expects a reply
  pub fn notice( &mut self, target : &str, message : &str ) {
    let sendline = format! ( "NOTICE {} :{}", target, message );
    self.send_str( sendline.as_slice( ) );
  }
  
  /// `action` sends a CTCP action message to the target
  ///
  /// # Arguments
  ///
  /// * `target` - target of the message
  /// * `message` - the action message
  ///
  /// # Notes
  ///
  /// * This is equivalent to doing "/me does an action" in a typical IRC client
  pub fn action( &mut self, target : &str, message : &str ) {
    let sendline = format! ( "ACTION {}", message );
    self.send_ctcp( target, sendline.as_slice( ) );
  }
  
  /// `join` joins a new channel
  ///
  /// # Arguments
  ///
  /// * `channel` - channel to join
  pub fn join( &mut self, channel : &str ) {
    let sendline = format! ( "JOIN {}", channel );
    self.send_str( sendline.as_slice( ) );
  }
  
  /// `part` leaves a channel you're in
  ///
  /// # Arguments
  ///
  /// * `channel` - channel to part from
  pub fn part( &mut self, channel : &str ) {
    let sendline = format! ( "PART {}", channel );
    self.send_str( sendline.as_slice( ) );
  }
  
  /// `nick` changes nickname on the server
  ///
  /// # Arguments
  ///
  /// * `nick` - nickname to change to
  pub fn nick( &mut self, nick : &str ) {
    let sendline = format! ( "NICK {}", nick );
    self.send_str( sendline.as_slice( ) );
  }
}