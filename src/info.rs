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