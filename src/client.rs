// import built in modules
use std::io;
use std::sync::mpsc;
use std::thread;

// import custom modules
use connection;
use info;
use message;
use reader;
use utils::debug;

pub struct Client {
  pub info    : info::IrcInfo,
  pub conn    : connection::ServerConnection,
  pub writer  : io::LineBufferedWriter < io::TcpStream >,
  
  thread      : Option < thread::Thread >,
  
  registered  : bool,
}

impl Client {
  pub fn connect ( 
    host : &str, 
    port : u16, 
    pass : &str, 
    info : info::IrcInfo
  ) -> Client
  {
    let mut conn : connection::ServerConnection = 
      connection::ServerConnection::connect( host, port, pass );
    let mut wrt = conn.spin_writer( );
    let mut client = Client {
      info        : info,
      conn        : conn,
      writer      : wrt,
      registered  : false,
      thread      : None,
    };
    
    client
  }
  
  fn close( &mut self ) {
    self.writer.write_line( "QUIT" );
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
    let msg = match message::Message::parse( s.as_slice() ) {
      Some ( m ) => m,
      None       => panic! ( "BAD" ),
    };
    
    // perform basic callbacks
    match msg.code.as_slice( ) {
      "PING"   => { 
        debug::info( "responding to ping request from server" );
        w.write_line( msg.pong( ).raw.as_slice( ) ); 
      },
      "NOTICE" => {
        if !*registered {
          debug::info( "registering on server" );
          let nickline = format! ( "NICK {}", i.nick_name );
          let userline = format! ( "USER {} * * :{}", 
            i.user_name, i.real_name );
            
          w.write_line( nickline.as_slice() );
          w.write_line( userline.as_slice() );
          *registered = true;
        }
      },
      "003"  => { 
        debug::info( "joining channels" );
        for chan in i.channels.iter() {
          let joinline = format! ( "JOIN {}", chan );
          w.write_line( joinline.as_slice( ) ); 
        }
      },
      _      => ()
    };
    
    // send the message back along our channel
    chan.send( msg );
  }
  
  fn handle_send( s : String, w : &mut io::LineBufferedWriter < io::TcpStream > ) {
    w.write_line( s.as_slice() ); 
    debug::disp( s.as_slice( ), false );
  }
  
  fn start_handler( 
    mut w : io::LineBufferedWriter < io::TcpStream >, 
    i : info::IrcInfo,
    mut chan : mpsc::Sender < message::Message >,
    port : mpsc::Receiver < connection::ConnEvent >
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
            break;
          },
        },
        Err ( e ) => {
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
  
  pub fn send_msg( &mut self, m : message::Message ) {
    match self.conn.chan.send( connection::ConnEvent::Send( m.raw ) ) {
      Ok ( _ )  => (),
      Err ( e ) => debug::err( "sending message to client", "" ),
    }
  }
  
  pub fn send_str( &mut self, s : &str ) {
    match self.conn.chan.send( connection::ConnEvent::Send( s.to_string( ) ) ) {
      Ok ( _ )  => (),
      Err ( e ) => debug::err( "sending message to client", "" ),
    }
  }
  
  pub fn start_thread ( mut self ) -> ( mpsc::Receiver < message::Message >, Client )  {
    debug::oper( "starting client thread..." );
    let (tx,rx) = mpsc::channel( );
    let params  = ( self.conn.tcp.clone( ), self.conn.chan.clone( ), self.conn.spin_writer( ), self.info.clone( ), self.conn.listen.expect( "no receiver found" ) );
    self.thread = Some( thread::Thread::spawn( move || {
      Client::start_reader( params.0, params.1 );
      Client::start_handler( params.2, params.3, tx.clone( ), params.4 );
    } ) );
    self.conn.listen = None;
    ( rx, self )
  }
  
  pub fn stop( &mut self ) {
    match self.conn.chan.send( connection::ConnEvent::Abort( 
      String::from_str( "stop called from client" ) ) ) {
      Ok ( _ )  => (),
      Err ( e ) => debug::err( "stopping client", "" ),
    }
  }
}

// fn main() {
  // let inf = info::IrcInfo {
    // nick_name : "ReturnOfBot".to_string( ),
    // user_name : "ReturnOfBot".to_string( ),
    // real_name : "I'm back, baby".to_string( ),
    // channels : vec! [ "#thefuture".to_string( ) ],
  // };
  // let clnt = Client::connect( "91.217.189.76", 6667, "", inf );
  
  // let rx = clnt.start_thread( );
  
  // for msg in rx.iter( ) {
    // debug::disp( msg.raw.as_slice( ), true );
  // }
// }