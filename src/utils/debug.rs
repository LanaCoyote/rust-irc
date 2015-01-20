use std::fmt;

pub enum Level {
  Err,
  Warn,
  Oper,
  Info,
  DispIn,
  DispOut,
}

#[allow(dead_code)]
pub fn err <T,S> ( s : T, e : S ) where T : fmt::String, S : fmt::String {
  let data = format! ( "err in {} : {}", s, e );
  log( Level::Err, data );
}

#[allow(dead_code)]
pub fn warn <T,S> ( s : T, w : S ) where T : fmt::String, S : fmt::String {
  let data = format! ( "warning in {} : {}", s, w );
  log( Level::Warn, data );
}

#[allow(dead_code)]
pub fn oper <T> ( s : T ) where T : fmt::String {
  log( Level::Oper, s );
}

#[allow(dead_code)]
pub fn info <T> ( s : T ) where T : fmt::String {
  log( Level::Info, s );
}

#[allow(dead_code)]
pub fn disp <T> ( s : T, inc : bool ) where T : fmt::String {
  match inc {
    true  => log( Level::DispIn, s ),
    false => log( Level::DispOut, s ),
  }
}

fn log <T> ( l : Level, s : T ) where T : fmt::String {
  match l {
    Level::Err     => println!( "!! {}", s ),
    Level::Warn    => println!( "** {}", s ),
    Level::Oper    => println!( "$$ {}", s ),
    Level::Info    => println!( " ~ {}", s ),
    Level::DispIn  => println!( " > {}", s ),
    Level::DispOut => println!( " < {}", s ),
  }
}