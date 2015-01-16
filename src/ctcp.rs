static X_DELIM : char                             = '\x01';
static M_CNVRT : [(&'static str,&'static str); 4] = [ ("\x00","\x140"),
                                                      ("\n","\x14n"),
                                                      ("\r","\x14r"),
                                                      ("\x14","\x14\x14") ];
static X_CNVRT : [(&'static str,&'static str); 2] = [ ("\x01","\\a"),
                                                      ("\\","\\\\") ];

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