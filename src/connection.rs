use std::io;

pub enum Event {
  Send( String ),
  Receive( String ),
}

pub enum Result {
  Ok,
  Error( String ),
}

pub struct ServerConnection {
  pub tcp       : io::TcpStream,
  pub host      : String,
  pub port      : u16,
  pub sx        : Sender <Event>,
  pub rx        : Receiver <Event>,
  pub connected : bool,
}

impl ServerConnection {
  pub fn new( host : &str, port : u16 ) -> ServerConnection {
    let address = format! ( "{}:{}", host, port );
    let tcp = match io::TcpStream::connect( addr[] ) {
      Ok( x )  => x,
      Err( e ) => {
        output::panic( "in new ServerConnection", e );
      },
    }
    output::info( format! ( "connected to {}:{}", host, port ) );
    
    let ( tx, rx ) = channel();
    
    ServerConnection {
      tcp   : tcp,
      host  : host.to_string(),
      port  : port,
      tx    : tx,
      rx    : rx,
    }
  }
  
  pub fn close( mut self ) {
    match self.tcp.close_read() {
      Err(e)  => output::error( "closing read", e ),
      _       => (),
    };
    
    match self.tcp.close_write() {
      Err(e)  => output::error( "closing write", e ),
      _       => (),
    };
    
    drop( self.tcp.clone() );
  }
  
  pub fn write( &self, s : &String ) {
    let s = s[];
    output::msg( s );
    _write_line( 
  }
  
}

fn _write_line( 
  stream : &mut io::LineBufferedWriter < TcpStream >,
  line   : &str
) {
  match stream.write_line( line ) {
    Err(e)  => output::error( "writing line", e ),
    _       => (),
  }
}

fn _read_line(
  stream : &mut io::BufferedReader < TcpStream >
) -> Option < String > {
  match stream.read_line() {
    Ok (x) => Some(x),
    Err (x) => {
      output::error( "reading line", e );
      None
    },
  }
}