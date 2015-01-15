// import built in modules
use std::io;
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
  pub info    : info::IrcInfo,
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
    info : info::IrcInfo
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
  
  fn close( &mut self ) {
    self.conn.close( );
  }
  
  fn handle_recv( 
    s : String,                                        // raw message received
    w : &mut io::LineBufferedWriter < io::TcpStream >, // writer to output to
    i : &info::IrcInfo,                                // irc client info
    registered : &mut bool,                            // are we registered?
    chan : &mut mpsc::Sender < message::Message >      // channel to send msg on
  ) {
    // parse our raw string into a usable message
    let msg = match message::Message::parse( 
      ctcp::ctcp_dequote( ctcp::low_level_dequote( s.clone( ) ) ).as_slice( ) ) {
      Some ( m ) => m,
      None       => {
        debug::err( "parsing IRC message", "message is not an IRC message" );
        debug::info( s.clone( ).as_slice( ) );
        return;
      },
    };
    
    // perform basic callbacks
    match msg.code.as_slice( ) {
      "PING"   => { 
        debug::info( "responding to ping request from server..." );
        match w.write_line( msg.pong( ).raw.as_slice( ) ) {
          Ok ( _ )  => (),
          Err ( e ) => debug::err( "ping response", e.desc ),
        };
      },
      "NOTICE" => {
        if !*registered {
          debug::info( "registering on server..." );
          let nickline = format! ( "NICK {}", i.nick_name );
          let userline = format! ( "USER {} * * :{}", 
            i.user_name, i.real_name );
            
          match w.write_line( nickline.as_slice() ) {
            Ok ( _ )  => debug::info( "registering nick on server" ),
            Err ( e ) => debug::err( "nick registration", e.desc ),
          };
          match w.write_line( userline.as_slice() ) {
            Ok ( _ )  => debug::info( "registering username on server" ),
            Err ( e ) => debug::err( "username registration", e.desc ),
          };
          *registered = true;
        }
      },
      "003"  => { 
        debug::info( "joining channels..." );
        for chan in i.channels.iter() {
          let joinline  = format! ( "JOIN {}", chan );
          let debugline = format! ( "joining channel {}", chan );
          match w.write_line( joinline.as_slice( ) ) {
            Ok ( _ )  => debug::info( debugline.as_slice( ) ),
            Err ( e ) => debug::err( debugline.as_slice( ), e.desc ),
          };
        }
      },
      _      => ()
    };
    
    // send the message back along our channel
    match chan.send( msg ) {
      Ok ( _ )  => (),
      Err ( _ ) => debug::err( "returning message to user", "" ),
    }
  }
  
  fn handle_send( s : String, w : &mut io::LineBufferedWriter < io::TcpStream > ) {
    match w.write_line( ctcp::low_level_quote( 
      ctcp::ctcp_quote( s.clone( ) ) ).as_slice( ) ) {
      Ok ( _ )  => (),
      Err ( e ) => debug::err( "writing sent message", e.desc ),
    }
    debug::disp( s.as_slice( ), false );
  }
  
  fn start_handler( 
    mut w : io::LineBufferedWriter < io::TcpStream >, // writer to send messages to
    i : info::IrcInfo,                                // client info
    mut chan : mpsc::Sender < message::Message >,     // channel to send received messages over
    port : mpsc::Receiver < connection::ConnEvent >   // port to receive data on
  ) {
    debug::oper( "starting message handler..." );
    let mut registered = false;
    loop {
      match port.recv( ) {
        Ok ( t )  => match t {
          connection::ConnEvent::Send( s ) => Client::handle_send( s, &mut w ),
          connection::ConnEvent::Recv( s ) => Client::handle_recv( s, &mut w, &i, &mut registered, &mut chan ),
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
  pub fn start_thread ( mut self ) -> ( mpsc::Receiver < message::Message >, Client )  {
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
  }
  
  /// `stop` ends the client thread.
  pub fn stop( &mut self ) {
    match self.conn.chan.send( connection::ConnEvent::Abort( 
      String::from_str( "stop called from client" ) ) ) {
      Ok ( _ )  => self.close( ),
      Err ( _ ) => debug::err( "stopping client", "" ),
    }
  }
}