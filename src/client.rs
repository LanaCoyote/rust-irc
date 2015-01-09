#![feature(slicing_syntax)]
extern crate regex;

use std::io;

use utils::debug;

mod connection;
mod info;
mod message;
mod reader;
mod utils;
mod writer;

pub struct Client <'cl> {
  pub info    : info::IrcInfo,
  pub conn    : connection::ServerConnection <'cl>,
  pub writer  : io::LineBufferedWriter < io::TcpStream >,
  
  registered  : bool,
}

impl <'cl> Client <'cl> {
  pub fn connect <'a> ( 
    host : &str, 
    port : u16, 
    pass : &str, 
    info : info::IrcInfo
  ) -> Client <'a>
  {
    let mut conn : connection::ServerConnection<'a> = connection::ServerConnection::connect( host, port, pass );
    let mut wrt = conn.spin_writer( );
    let mut client = Client {
      info        : info,
      conn        : conn,
      writer      : wrt,
      registered  : false,
    };
    
    client
  }
  
  fn close( &mut self ) {
    self.writer.write_line( "QUIT" );
    self.conn.close( );
  }
  
  fn handle_recv( &mut self, s : String ) {
    let msg = match message::Message::parse( s.as_slice() ) {
      Some ( m ) => m,
      None       => panic! ( "BAD" ),
    };
    
    match msg.code.as_slice( ) {
      "PING"   => { self.writer.write_line( msg.pong( ).raw.as_slice( ) ); },
      "NOTICE" => {
        if !self.registered {
          let nickline = format! ( "NICK {}", self.info.nick_name );
          let userline = format! ( "USER {} * * :{}", 
            self.info.user_name, self.info.real_name );
            
          self.writer.write_line( nickline.as_slice() );
          self.writer.write_line( userline.as_slice() );
          self.registered = true;
        }
      },
      "003"  => { self.writer.write_line( "JOIN #thefuture" ); },
      _      => ()
    };
    
    debug::disp( s.as_slice( ), true );
  }
  
  fn handle_send( &mut self, s : String ) {
    self.writer.write_line( s.as_slice() ); 
    debug::disp( s.as_slice( ), false );
  }
  
  fn start( &mut self ) {
    self.start_reader( );
    self.start_handler( );
    self.close( );
  }
  
  fn start_handler( &mut self ) {
    for ev in self.conn.listen.iter( ) {
      match ev {
        connection::ConnEvent::Send( s ) => self.handle_send( s ),
        connection::ConnEvent::Recv( s ) => self.handle_recv( s ),
        connection::ConnEvent::Abort( s ) => break,
      }
    }
  }
  
  fn start_reader( &self ) {
    let tcp   = self.conn.tcp.clone();
    let chan  = self.conn.chan.clone();
    
    let rthread = std::thread::Thread::spawn( move || {
      let mut rdr = reader::IrcReader::new( tcp, chan );
      rdr.start( );
    } );
  }
}

fn main() {
  let inf = info::IrcInfo {
    nick_name : "ReturnOfBot".to_string( ),
    user_name : "ReturnOfBot".to_string( ),
    real_name : "I'm back, baby".to_string( ),
    channels : vec! [ "#thefuture".to_string( ) ],
  };
  let mut clnt = Client::connect( "91.217.189.76", 6667, "", inf );
  let mut isreg = false;
  
  clnt.start( );
}