#![feature(slicing_syntax)]
extern crate regex;

use regex::Regex;

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
      Err( e  ) => panic! ( "error creating regex parser: {}", e ),
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
  
  /// `param` gets the parameter at index `num`
  ///
  /// # Arguments
  ///
  /// `num` - the index of the parameter to get, 1-based
  ///
  /// # Returns
  ///
  /// The parameter at the given index or None if that parameter doesn't exist
  pub fn param( &self, num : uint ) -> Option < &str > {
    let re = match Regex::new( r"(:.*|\S+)" ) {
      Ok ( re ) => re,
      Err( e  ) => panic! ( "error creating regex parser: {}", e ),
    };
    match re.captures( self.params.as_slice( ) ) {
      Some ( cap )  => cap.at( num ),
      None          => None,
    }
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
  
  /// `trailing` gets the trailing parameter of a message (the last one)
  ///
  /// # Returns
  ///
  /// The trailing parameter or None if the message has no trailing parameter
  pub fn trailing( &self ) -> Option < &str > {
    let re = match Regex::new( r":.*" ) {
      Ok ( re ) => re,
      Err( e  ) => panic! ( "error creating regex parser: {}", e ),
    };
    match re.find( self.params.as_slice( ) ) {
      Some( ( start, end ) )  => Some( self.params.as_slice( )[start+1..end] ),
      None                    => None,
    }
  }
}

impl std::fmt::Show for Message {
  fn fmt( &self, f : &mut std::fmt::Formatter ) -> std::fmt::Result {
    write! ( f, "{}", self.raw.as_slice( ) )
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