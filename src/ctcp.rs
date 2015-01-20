use regex;

use message;
use utils::debug;

static X_DELIM : char                             = '\x01';
static M_CNVRT : [(&'static str,&'static str); 4] = [ ("\x14","\x14\x14"),
                                                      ("\x00","\x140"),
                                                      ("\n","\x14n"),
                                                      ("\r","\x14r") ];
static X_CNVRT : [(&'static str,&'static str); 2] = [ ("\\","\\\\"),
                                                      ("\x01","\\a") ];

/// `CtcpRequest` is an abstraction of a CTCP command.
///
/// # Members
///
/// * `command` - command string of the CTCP request. e.g. "ACTION"
/// * `params` - parameters of the request as a space delimited string
pub struct CtcpRequest {
  pub command : String,
  pub params  : String,
}

impl CtcpRequest {
  /// `new` creates a new CTCP request from a command and params
  ///
  /// # Arguments
  ///
  /// `cmd` - command to create the new request with
  /// `params` - parameters to create the new request with
  ///
  /// # Returns
  ///
  /// A CTCP request struct with the command and parameters as specified
  pub fn new ( cmd : String, pms : String ) -> CtcpRequest {
    CtcpRequest {
      command : cmd,
      params  : pms,
    }
  }
  
  pub fn quote ( &self ) -> CtcpRequest {
    CtcpRequest {
      command : ctcp_quote( self.command.clone( ) ),
      params  : ctcp_quote( self.params.clone( ) ),
    }
  }
  
  pub fn dequote ( &self ) -> CtcpRequest {
    CtcpRequest {
      command : ctcp_dequote( self.command.clone( ) ),
      params  : ctcp_dequote( self.params.clone( ) ),
    }
  }
  
  /// `param` gets a particular parameter from a CTCP request
  ///
  /// # Arguments
  ///
  /// * `num` - the number of the parameter to get
  ///
  /// # Returns
  ///
  /// The parameter if it exists, otherwise None
  ///
  /// # Notes
  ///
  /// * The 0th parameter is the command of a CTCP request.
  pub fn param ( &self, num : usize ) -> Option < &str > {
    if num == 0 { return Some( self.command.as_slice( ) ) };
    let pms : Vec < &str > = self.params.as_slice( ).split_str( " " ).collect( );
    if num > pms.len( ) {
      None
    } else {
      Some( pms[ num-1 ] )
    }
  }
}

impl Clone for CtcpRequest {
  fn clone ( &self ) -> CtcpRequest {
    CtcpRequest {
      command : self.command.clone( ),
      params  : self.params.clone( ),
    }
  }
}

impl ToString for CtcpRequest {
  fn to_string ( &self ) -> String {
    if self.params.len( ) > 0 {
      format! ( "{}{} {}{}", X_DELIM, self.command, self.params, X_DELIM )
    } else {
      format! ( "{}{}{}", X_DELIM, self.command, X_DELIM )
    }
  }
}

/// `low_level_quote` wraps a message for transmission by obscuring low-level
/// characters.
///
/// # Arguments
///
/// * `trail` - message string to quote
///
/// # Returns
///
/// A String with low-level characters quoted out for proper IRC transmission
pub fn low_level_quote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in M_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.0, conv.1 );
  }
  newtrail
}

/// `low_level_dequote` reverses the quote process and unhides low-level
/// characters.
///
/// # Arguments
///
/// * `trail` - quoted string to dequote
///
/// # Returns
///
/// The original String before low-level quoting
pub fn low_level_dequote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in M_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.1, conv.0 );
  }
  newtrail
}

/// `ctcp_quote` obscures ctcp tags in a message for transmission
///
/// # Arguments
///
/// * `trail` - message string to quote
///
/// # Returns
///
/// A String with CTCP tags obscured for extraction
pub fn ctcp_quote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in X_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.0, conv.1 );
  }
  newtrail
}

/// `ctcp_dequote` reverses the CTCP quote and gets internal CTCP tags back
///
/// # Arguments
///
/// * `trail` - quoted string to dequote
///
/// # Returns
///
/// The original string with all CTCP delimiters returned
pub fn ctcp_dequote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in X_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.1, conv.0 );
  }
  newtrail
}

/// `tag` wraps a string in CTCP tag delimiters
///
/// # Arguments
///
/// * `s` - string slice to be wrapped
///
/// # Returns
///
/// The original string as a CTCP tag
pub fn tag ( s : &str ) -> String {
  let mut newstring = String::from_str( s );
  newstring.insert( 0, X_DELIM );
  newstring.push( X_DELIM );
  newstring
}

/// `has_tag` returns true if a string contains the given CTCP tag
///
/// # Arguments
///
/// * `s` - string to search for the tag in
/// * `t` - CTCP tag to search for, e.g. "ACTION"
///
/// # Returns
///
/// `true` if the string contains the given tag, otherwise false
pub fn has_tag ( s : &str, t : &str ) -> bool {
  let patt = format! ( "{}{} ?([^{}]*){}", X_DELIM, t, X_DELIM, X_DELIM );
  let re   = match regex::Regex::new( patt.as_slice( ) ) {
    Ok ( re ) => re,
    Err ( e ) => {
      debug::err( "creating ctcp regex", e.msg );
      return false;
    },
  };
  re.is_match( s )
}

/// `get_tag` gets a CTCP request from a string if it exists
///
/// # Arguments
///
/// * `s` - string to search for the tag in
/// * `t` - CTCP tag to search for, e.g. "ACTION"
///
/// # Returns
///
/// The CTCP request with parameters in the given string if the tag exists,
/// otherwise None
pub fn get_tag ( s : &str, t : &str ) -> Option < CtcpRequest > {
  let patt = format! ( "{}{} ?([^{}]*){}", X_DELIM, t, X_DELIM, X_DELIM );
  let re   = match regex::Regex::new( patt.as_slice( ) ) {
    Ok ( re ) => re,
    Err ( e ) => {
      debug::err( "creating ctcp regex", e.msg );
      return None;
    },
  };
  match re.captures( s ) {
    Some( cap ) => Some ( CtcpRequest::new( t.to_string( ), match cap.at( 1 ) {
      Some( pms ) => String::from_str( pms ),
      None        => String::new( ),
    } ) ),
    None        => None,
  }
}

/// `combine` combines a string with a CTCP request
///
/// # Arguments
///
/// * `s` - string to combine request with
/// * `cmd` - CTCP request to insert into the string
///
/// # Returns
///
/// A String with the CTCP request appended to it
pub fn combine ( s : String, cmd : CtcpRequest ) -> String {
  format! ( "{}{}", s, cmd.to_string( ).as_slice( ) )
}

/// `combine_msg` combines a message struct with a CTCP request
///
/// # Arguments
///
/// * `msg` - message to combine the request into
/// * `cmd` - CTCP request to insert into the message
///
/// # Returns
///
/// A Message struct with the CTCP request appended to it
///
/// # Notes
///
/// * Only PRIVMSG or NOTICE based messages can contain a CTCP request. If any
/// other message is passed in, a warning will be given and the original
/// message returns unaltered.
pub fn combine_msg( msg : message::Message, cmd : CtcpRequest ) -> message::Message {
  if !msg.is_message( ) {
    debug::warn( "ctcp combine message", 
      "CTCP requests can only be combined with a PRIVMSG or NOTICE based message" );
    return msg;
  }
  let newpms = combine( msg.params, cmd );
  message::Message::new( msg.source, msg.code.as_slice( ), newpms.as_slice( ) )
}

/// `parse_cmd` takes a CTCP tag and creates a CTCP request struct from it
///
/// # Arguments
///
/// * `s` - contents of the CTCP tag to parse
///
/// # Returns
///
/// A CTCP request that represents the given tag
fn parse_cmd ( s : String ) -> CtcpRequest {
  let data : Vec < &str > = s.as_slice( ).split_str( " " ).collect( ); // break up the params
  let cmd : &str = data[0];         // the first word is the command
  let mut params = String::new( );  // the string of parameters
  
  // loop through all our remaining parameters and push them to the param str
  for param in data.iter( ) {
    if param.as_slice( ) == cmd { continue };
    params.push_str( *param );
    params.push( ' ' );
  }
  
  if params.len( ) > 0 {
    params.pop( );
  }
  
  // construct our request
  CtcpRequest::new( cmd.to_string( ), params )
}

/// `extract` gets all the CTCP requests from a string
///
/// # Arguments
///
/// `s` - the string slice to extract requests from
///
/// # Returns
/// 
/// A tuple containing:
/// * The original string with all CTCP requests removed
/// * A vector of CTCP requests
pub fn extract ( s : &str ) -> ( String, Vec < CtcpRequest > ) {
  let mut cmds : Vec < CtcpRequest > = Vec::new( ); // vec of cmds
  let mut newstring = String::new( ); // the parsed and extracted string
  let mut section   = String::new( ); // the current section being read
  let mut intag     = false;          // are we in a ctcp tag?
  
  // loop through our string
  for ch in s.chars( ) {
    // if we hit a ctcp tag delimiter, push the section to the right place
    if ch == X_DELIM {
      match intag {
        false => newstring.push_str( section.as_slice( ) ),
        true  => cmds.push( parse_cmd( section.clone( ) ) ),
      }
      section.clear( );
      intag = !intag;
    // otherwise continue parsing the section
    } else {
      section.push( ch );
    }
  }
  
  // if we still have a section left over (perhaps from an unclosed ctcp tag),
  // push it to the parsed string
  if !section.is_empty( ) {
    newstring.push_str( section.as_slice( ) );
  }
  
  // construct our tuple
  ( newstring, cmds )
}

/// `extract_msg` gets all the CTCP requests from a Message struct
///
/// # Arguments
///
/// * `msg` - the message struct to extract requests from
///
/// # Returns
///
/// A tuple containing:
/// * The original message with all CTCP requests removed from its body
/// * A vector of CTCP requests
///
/// # Notes
///
/// * CTCP requests can only be extracted from a PRIVMSG or NOTICE based
/// message. All other messages will throw a warning and return the original
/// message back with no CTCP requests.
pub fn extract_msg ( msg : message::Message ) -> ( message::Message, Vec < CtcpRequest > ) {
  if !msg.is_message( ) {
    debug::warn( "ctcp extract message",
      "CTCP requests can only be extracted from a NOTICE or PRIVMSG based message" );
    return ( msg, Vec::new( ) );
  }
  
  // extract the commands from the message and rebuild it
  let ( newparams, cmds ) = extract( msg.params.as_slice( ) );
  let newmsg = message::Message::new( msg.source, msg.code.as_slice( ), 
    newparams.as_slice( ) );
    
  // construct our tuple
  ( newmsg, cmds )
}

// ** TEST MODULE ************************************************************
mod test {
  #[allow(unused_imports)]
  use message::Message;

  #[test]
  fn test_llquote () {
    let quote1 = super::low_level_quote( "To shreds, you say?".to_string( ) );
    let quote2 = super::low_level_quote( "To be, or not to be\nThat is the question".to_string( ) );
    let quote3 = super::low_level_quote( "Behold the M_QUOTE: \x14".to_string( ) );
    assert! ( quote1 == "To shreds, you say?" );
    assert! ( quote2 == "To be, or not to be\x14nThat is the question" );
    assert! ( quote3 == "Behold the M_QUOTE: \x14\x14" );
  }
  
  #[test]
  fn test_lldequote () {
    let mut quote1 = super::low_level_quote( "To shreds, you say?".to_string( ) );
    let mut quote2 = super::low_level_quote( "To be, or not to be\nThat is the question".to_string( ) );
    let mut quote3 = super::low_level_quote( "Behold the M_QUOTE: \x14".to_string( ) );
    quote1 = super::low_level_dequote( quote1 );
    quote2 = super::low_level_dequote( quote2 );
    quote3 = super::low_level_dequote( quote3 );
    assert! ( quote1 == "To shreds, you say?" );
    assert! ( quote2 == "To be, or not to be\nThat is the question" );
    assert! ( quote3 == "Behold the M_QUOTE: \x14" );
  }
  
  #[test]
  fn test_tag () {
    let tagme = super::tag( "This is tagged" );
    assert! ( tagme == "\x01This is tagged\x01" );
    assert! ( super::has_tag( tagme.as_slice( ), "This" ) );
    assert! ( super::get_tag( tagme.as_slice( ), "This" ).unwrap( ).params.as_slice( ) == "is tagged" );
    let message = String::from_str( "I'm the message" );
    let request = super::CtcpRequest::new( "ACTION".to_string( ), "says hello".to_string( ) );
    let combined = super::combine( message, request.clone( ) );
    assert! ( super::has_tag( combined.as_slice( ), "ACTION" ) );
    assert! ( super::get_tag( combined.as_slice( ), "ACTION" ).unwrap( ).to_string( ) == request.to_string( ) );
  }
  
  #[test]
  fn test_ctcprequest () {
    let request = super::CtcpRequest::new( "ACTION".to_string( ), "says hello".to_string( ) );
    assert! ( request.command.as_slice( ) == "ACTION" );
    assert! ( request.params.as_slice( ) == "says hello" );
    assert! ( request.param( 0 ).unwrap( ) == "ACTION" );
    assert! ( request.param( 1 ).unwrap( ) == "says" );
    assert! ( request.param( 2 ).unwrap( ) == "hello" );
    assert! ( request.param( 3 ) == None );
    assert! ( request.to_string( ).as_slice( ) == super::tag( "ACTION says hello" ) );
  }
  
  #[test]
  fn test_combine_extract_strs () {
    let initial = String::from_str( "I'm the message" );
    let request = super::CtcpRequest::new( "ACTION".to_string( ), "says hello".to_string( ) );
    let request2 = super::CtcpRequest::new( "USERINFO".to_string( ), String::new( ) );
    let combined = super::combine( super::combine( initial.clone( ), request.clone( ) ), request2.clone( ) );
    assert! ( combined.as_slice( ).contains( initial.as_slice( ) ) );
    assert! ( combined.as_slice( ).contains( request.to_string( ).as_slice( ) ) );
    assert! ( combined.as_slice( ).contains( request2.to_string( ).as_slice( ) ) );
    let (ext,rqs) = super::extract( combined.as_slice( ) );
    assert! ( ext == initial );
    assert! ( rqs[0].to_string( ) == request.to_string( ) );
    assert! ( rqs[1].to_string( ) == request2.to_string( ) );
  }
  
  #[test]
  fn test_combine_extract_msgs () {
    let initial = Message::parse( "PRIVMSG I'm the message" ).unwrap( );
    let request = super::CtcpRequest::new( "ACTION".to_string( ), "says hello".to_string( ) );
    let request2 = super::CtcpRequest::new( "USERINFO".to_string( ), String::new( ) );
    let combined = super::combine_msg( super::combine_msg( initial.clone( ), request.clone( ) ), request2.clone( ) );
    assert! ( combined.raw.as_slice( ).contains( initial.raw.as_slice( ) ) );
    assert! ( combined.raw.as_slice( ).contains( request.to_string( ).as_slice( ) ) );
    assert! ( combined.raw.as_slice( ).contains( request2.to_string( ).as_slice( ) ) );
    let (ext,rqs) = super::extract_msg( combined );
    assert! ( ext.raw == initial.raw );
    assert! ( rqs[0].to_string( ) == request.to_string( ) );
    assert! ( rqs[1].to_string( ) == request2.to_string( ) );
    let failmsg = Message::parse( "JOIN #failville" ).unwrap( );
    let combined2 = super::combine_msg( failmsg.clone( ), request.clone( ) );
    assert! ( failmsg.raw == combined2.raw );
  }
}