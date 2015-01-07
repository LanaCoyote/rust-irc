use config;

pub struct IrcInfo <'info> {
  pub nick_name : &'info str,
  pub user_name : &'info str,
  pub real_name : &'info str,
  
  pub channels  : Vec<&'info str>,
}

impl <'info> IrcInfo <'info> {
  pub fn new <'a> ( cfg : config::IrcConfig ) -> IrcInfo<'a> {
    IrcInfo {
      nick_name : cfg.nick_name,
      user_name : cfg.user_name,
      real_name : cfg.real_name,
      channels  : cfg.channels,
    }
  }
}