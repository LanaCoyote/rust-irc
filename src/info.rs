pub struct IrcInfo {
  pub nick_name : String,
  pub user_name : String,
  pub real_name : String,
  
  pub channels  : Vec < String >,
}

impl Clone for IrcInfo {
  fn clone( &self ) -> IrcInfo {
    IrcInfo {
      nick_name : self.nick_name.clone( ),
      user_name : self.user_name.clone( ),
      real_name : self.real_name.clone( ),
      channels  : self.channels.clone( ),
    }
  }
}

impl IrcInfo {
  pub fn gen( nick : &str, user : &str, real : &str, chans : Vec < &str > ) -> IrcInfo {
    let mut cvec : Vec < String > = Vec::new();
    for chan in chans.iter( ) {
      cvec.push( String::from_str( *chan ) );
    }
    IrcInfo {
      nick_name : String::from_str( nick ),
      user_name : String::from_str( user ),
      real_name : String::from_str( real ),
      channels  : cvec,
    }
  }
}