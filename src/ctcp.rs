static X_DELIM : char                             = '\x01';
static M_CNVRT : [(&'static str,&'static str); 4] = [ ("\x14","\x14\x14"),
                                                      ("\x00","\x140"),
                                                      ("\n","\x14n"),
                                                      ("\r","\x14r") ];
static X_CNVRT : [(&'static str,&'static str); 2] = [ ("\\","\\\\"),
                                                      ("\x01","\\a") ];

pub fn low_level_quote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in M_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.0, conv.1 );
  }
  newtrail
}

pub fn low_level_dequote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in M_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.1, conv.0 );
  }
  newtrail
}

pub fn ctcp_quote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in X_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.0, conv.1 );
  }
  newtrail
}

pub fn ctcp_dequote ( trail : String ) -> String {
  let mut newtrail = trail.clone( );
  for conv in X_CNVRT.iter( ) {
    newtrail = newtrail.replace( conv.1, conv.0 );
  }
  newtrail
}

pub fn tag ( s : &str ) -> String {
  let mut newstring = String::from_str( s );
  newstring.insert( 0, X_DELIM );
  newstring.push( X_DELIM );
  newstring
}

// ** TEST MODULE ************************************************************
mod test {
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
  }
}