pub enum Level {
  Err,
  Warn,
  Oper,
  Info,
  DispIn,
  DispOut,
}

#[allow(dead_code)]
pub fn err ( s : &str, e : &str ) {
  let data = format! ( "err in {} : {}", s, e );
  log( Level::Err, data );
}

#[allow(dead_code)]
pub fn warn ( s : &str, w : &str ) {
  let data = format! ( "warning in {} : {}", s, w );
  log( Level::Warn, data );
}

#[allow(dead_code)]
pub fn oper ( s : &str ) {
  log( Level::Oper, s.to_string( ) );
}

#[allow(dead_code)]
pub fn info ( s : &str ) {
  log( Level::Info, s.to_string( ) );
}

#[allow(dead_code)]
pub fn disp ( s : &str, inc : bool ) {
  match inc {
    true  => log( Level::DispIn, s.to_string( ) ),
    false => log( Level::DispOut, s.to_string( ) ),
  }
}

fn log ( l : Level, s : String ) {
  match l {
    Level::Err     => println!( "!! {}", s ),
    Level::Warn    => println!( "** {}", s ),
    Level::Oper    => println!( "$$ {}", s ),
    Level::Info    => println!( " ~ {}", s ),
    Level::DispIn  => println!( " > {}", s ),
    Level::DispOut => println!( " < {}", s ),
  }
}