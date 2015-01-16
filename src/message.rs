use regex::Regex;

use utils::debug;

/// `Source` abstracts the source of an IRC message
///
/// # Options
///
/// `Sender ( s : String )` - this message was sent by s
/// `None` - this message doesn't have a source, e.g. it came from us
pub enum Source {
  Sender ( String ),
  None,
}

impl Clone for Source {
  fn clone( &self ) -> Source {
    match *self {
      Source::Sender ( ref snd ) => Source::Sender( snd.clone( ) ),
      Source::None               => Source::None,
    }
  }
}

/// `Direction` specifies the flow of the IRC message
///
/// # Options
///
/// `Incoming` - message was sent from the server to the client
/// `Outgoing` - message is being sent to the server from the client
pub enum Direction {
  Incoming,
  Outgoing,
}

impl Copy for Direction {}

/// `Message` refers to an IRC message
/// 
/// # Members
///
/// `dir` - the direction of the message flow
/// `source` - the source of this message
/// `code` - the code associated with the message action
/// `params` - the message parameters
/// `raw` - the original message without formatting and parsing
pub struct Message {
  pub dir     : Direction,
  pub source  : Source,
  pub code    : String,
  pub params  : String,
  pub raw     : String,
}

impl Message {
  /// `new` creates a Message struct from a set of data
  ///
  /// # Arguments
  ///
  /// `source` - the source of the message (probably None)
  /// `code` - the code associated with this message action
  /// `params` - the parameters of this message as a string slice
  ///
  /// # Returns
  ///
  /// A formatted and ready Message struct
  ///
  /// # Notes
  ///
  /// - The direction of a message created by `new` is always `Outgoing`
  pub fn new( source : Source, code : &str, params : &str ) -> Message {
    Message {
      dir     : Direction::Outgoing,
      source  : source.clone( ),
      code    : code.to_string( ),
      params  : params.to_string( ),
      raw     : raw_from_data( source, code, params ),
    }
  }

  /// `parse` creates a new message struct from an unparsed IRC message
  ///
  /// # Arguments
  ///
  /// `msg` - the IRC message to parse for data
  ///
  /// # Returns
  ///
  /// Either a properly parsed and formatted Message struct or None
  ///
  /// # Notes
  ///
  /// - The direction of a message created by `parse` is always `Incoming`
  pub fn parse( msg : &str ) -> Option < Message > {
    let re      = match Regex::new( r"^(:\S+)?\s*(\S+)\s+(.*)\r?$" ) {
      Ok ( re ) => re,
      Err( e  ) => {
        debug::err( "creating message parser", e.msg.as_slice( ) );
        return None;
      },
    };
    let capture = re.captures( msg );
    
    match capture {
      Some ( cap )  => {
        Some( Message {
          dir     : Direction::Incoming,
          source  : match cap.at( 1 ) {
            None        => Source::None,
            Some( src ) => Source::Sender( src.to_string( ) ),
          },
          code    : match cap.at( 2 ) {
            None        => "".to_string( ),
            Some( cod ) => cod.to_string( ),
          },
          params  : match cap.at( 3 ) {
            None        => "".to_string( ),
            Some( prm ) => prm.to_string( ),
          },
          raw     : msg.to_string( ),
        } )
      },
      None          => None,
    }
  }
  
  /// `privmsg` generates a PRIVMSG message that goes to the target
  ///
  /// # Arguments
  ///
  /// `target` - the destination of the message, a nick, host, or channel
  /// `message` - the message body
  ///
  /// # Returns
  ///
  /// A message ready to be sent to the target
  pub fn privmsg( target : &str, message : &str ) -> Message {
    let params = format! ( "{} :{}", target, message );
    Message {
      dir     : Direction::Outgoing,
      source  : Source::None,
      code    : String::from_str( "PRIVMSG" ),
      params  : params.clone( ),
      raw     : raw_from_data( Source::None, "PRIVMSG", params.as_slice( ) ),
    }
  }
  
  /// `is_message` returns whether a message is a PRIVMSG based message
  ///
  /// # Returns
  ///
  /// - `true` if the message code is PRIVMSG or NOTICE
  /// - `false` otherwise
  pub fn is_message( &self ) -> bool {
    match self.code.as_slice( ) {
      "PRIVMSG" | "NOTICE" => true,
      _                   => false,
    }
  }
  
  /// `is_public` returns whether a message is a private message
  ///
  /// # Returns
  ///
  /// - `true` if the destination of the message is a channel
  /// - `false` otherwise (not a PRIVMSG or target is nick)
  pub fn is_public( &self ) -> bool {
    if !self.is_message( ) { return false };
    self.target( ).expect( "bad message" ).starts_with( "#" )
  }
  
  /// `nick` gets the nick or channel of the source, if there is one
  ///
  /// # Returns
  ///
  /// An Option enum with either the nick or channel of the source or None if it
  /// couldn't find one
  pub fn nick( &self ) -> Option < String > {
    match self.source.clone( ) {
      Source::Sender ( s ) => {
        let re = match Regex::new( r":([\w]+?)" ) {
          Ok ( re ) => re,
          Err ( e ) => {
            debug::err( "creating nick parser", e.msg.as_slice( ) );
            return None;
          },
        };
        match re.captures( s.as_slice() ) {
          Some ( cap ) => {
            match cap.at( 1 ) {
              Some ( res ) => Some( String::from_str( res ) ),
              None         => None,
            }
          },
          None         => None,
        }
      },
      Source::None         => None,
    }
  }
  
  /// `param` gets the parameter at index `num`
  ///
  /// # Arguments
  ///
  /// `num` - the index of the parameter to get, 1-based
  ///
  /// # Returns
  ///
  /// The parameter at the given index or None if that parameter doesn't exist
  pub fn param( &self, num : usize ) -> Option < &str > {
    let re = match Regex::new( r"(:.*|\S+)" ) {
      Ok ( re ) => re,
      Err( e  ) => {
        debug::err( "creating msg parameter parser", e.msg.as_slice( ) );
        return None;
      },
    };
    let mut i = 1;
    for cap in re.captures_iter( self.params.as_slice( ) ) {
      if i == num {
        return cap.at( 1 )
      }
      i += 1;
    }
    return None;
  }
  
  /// `pong` automatically reverses an incoming PING message
  ///
  /// # Returns
  ///
  /// A new Message struct formatted as a PONG response
  pub fn pong( &self ) -> Message {
    Message {
      dir     : Direction::Outgoing,
      source  : Source::None,
      code    : "PONG".to_string( ),
      params  : self.params.clone( ),
      raw     : raw_from_data( Source::None, "PONG", self.params.as_slice( ) ),
    }
  }
  
  /// `target` returns the target of a command
  ///
  /// # Returns
  ///
  /// - `Some` if the command has a target, containing the target
  /// - `None` if the command doesn't have a target
  pub fn target( &self ) -> Option < &str > {
    match self.code.as_slice( ) {
      "JOIN" | "PART" | "MODE" | "TOPIC" | "INVITE" | "PRIVMSG" | "NOTICE" | 
        "WHOIS" | "WHOWAS" | "KILL" | "PING" | "PONG" | "SUMMON" | "ISON" => 
        self.param( 1 ),
      "KICK" => self.param( 2 ),
      _      => self.param( 1 ),
    }
  }
  
  /// `trailing` gets the trailing parameter of a message (the last one)
  ///
  /// # Returns
  ///
  /// The trailing parameter or None if the message has no trailing parameter
  pub fn trailing( &self ) -> Option < &str > {
    let re = match Regex::new( r":(.*)" ) {
      Ok ( re ) => re,
      Err( e  ) => {
        debug::err( "creating msg trailing parser", e.msg.as_slice( ) );
        return None;
      },
    };
    match re.captures( self.params.as_slice( ) ) {
      Some( cap )  => cap.at( 1 ),
      None         => None,
    }
  }
}

impl Clone for Message {
  fn clone ( &self ) -> Message {
    Message {
      dir     : self.dir,
      source  : self.source.clone( ),
      code    : self.code.clone( ),
      params  : self.params.clone( ),
      raw     : self.raw.clone( ),
    }
  }
}

/// `raw_from_data` generates a raw message from a set of data
///
/// # Arguments
///
/// `source` - the source of the message
/// `code` - the code associated with the message action
/// `params` - parameters of the message
///
/// # Returns
///
/// A String that holds an unparsed IRC message
///
/// # Notes
///
/// - This helper function is used primarily for generating a message from
/// arbitrary data, such as in the `new` function.
fn raw_from_data( source : Source, code : &str, params : &str ) -> String {
  match source {
    Source::Sender( snd ) => {
      format! ( ":{} {} {}", snd, code, params ).to_string( )
    },
    Source::None          => format! ( "{} {}", code, params ).to_string( ),
  }
}

// ** TEST MODULE ************************************************************
mod test {
  #[test]
  fn test_newmessage () {
    let mymessage = super::Message::new( 
      super::Source::Sender( String::from_str( "Lancey" ) ),
      "PRIVMSG",
      "Detective :Hello!"
    );
    assert! ( mymessage.raw     == ":Lancey PRIVMSG Detective :Hello!" );
  }
  
  #[test]
  fn test_parsemessage () {
    let mymessage = super::Message::parse( ":Lancey PRIVMSG Detective :Hello!" ).unwrap( );
    assert! ( mymessage.code    == "PRIVMSG" );
    assert! ( mymessage.param( 1 ).unwrap( ) == "Detective" );
    assert! ( mymessage.params  == "Detective :Hello!" );
  }
  
  #[test]
  fn test_privmessage () {
    let mymessage = super::Message::privmsg( "Detective", "Hello!" );
    assert! ( mymessage.code    == "PRIVMSG" );
    assert! ( mymessage.param( 1 ).unwrap( ) == "Detective" );
    assert! ( mymessage.params  == "Detective :Hello!" );
    assert! ( mymessage.raw     == "PRIVMSG Detective :Hello!" );
  }
  
  #[test]
  fn test_ismessage () {
    let privmessage = super::Message::privmsg( "Detective", "Hello!" );
    let joinmessage = super::Message::new(
      super::Source::Sender( String::from_str( "Lancey" ) ),
      "JOIN",
      "#rust"
    );
    let nickmessage = super::Message::parse( ":Lancey NICK func_door" ).unwrap( );
    
    assert! ( privmessage.is_message( ) );
    assert! ( !joinmessage.is_message( ) );
    assert! ( !nickmessage.is_message( ) );
  }
  
  #[test]
  fn test_ispublic () {
    let pubmessage = super::Message::privmsg( "#rust", "Hello, everyone!" );
    let privmessage = super::Message::privmsg( "Detective", "Goodbye" );
    
    assert! ( pubmessage.is_public( ) );
    assert! ( !privmessage.is_public( ) );
  }
  
  #[test]
  fn test_nick () {
    let nonickmessage = super::Message::privmsg( "Detective", "Hello!" );
    let nickmessage = super::Message::parse( ":Detective JOIN #rust" ).unwrap( );
    
    assert! ( nickmessage.nick( ).is_some( ) );
    assert! ( nonickmessage.nick( ).is_none( ) );
  }
  
  #[test]
  fn test_param () {
    let mymessage = super::Message::parse( "PRIVMSG param1 param2 param3 :param4 param5" ).unwrap( );
    assert! ( mymessage.param( 1 ).unwrap( ) == "param1" );
    assert! ( mymessage.param( 2 ).unwrap( ) == "param2" );
    assert! ( mymessage.param( 3 ).unwrap( ) == "param3" );
    assert! ( mymessage.param( 4 ).unwrap( ) == ":param4 param5" );
    assert! ( mymessage.param( 5 ).is_none( ) );
  }
  
  #[test]
  fn test_pong () {
    let mymessage = super::Message::parse( "PING tolsun.oulu.fi" ).unwrap( );
    let pongmessage = mymessage.pong( );
    assert! ( pongmessage.code == "PONG" );
    assert! ( pongmessage.param( 1 ).unwrap( ) == "tolsun.oulu.fi" );
  }
  
  #[test]
  fn test_trailing () {
    let mymessage = super::Message::parse( "PRIVMSG param1 param2 param3 :param4 param5" ).unwrap( );
    let failmessage = super::Message::parse( "JOIN #rust" ).unwrap( );
    assert! ( mymessage.trailing( ).unwrap( ) == "param4 param5" );
    assert! ( failmessage.trailing( ).is_none( ) );
  }
  
  #[test]
  fn test_clone () {
    let mymessage = super::Message::privmsg( "Detective", "Hello!" );
    let clonemessage = mymessage.clone( );
    assert! ( mymessage.code == clonemessage.code );
    assert! ( mymessage.raw == clonemessage.raw );
  }
}
