/// `IrcConfig` is a struct used for setting up a new IRC client effectively
///
/// # Members
///
/// `host` - the host address to connect to
/// `port` - the port to connect to the server on
/// `nick_name` - the nickname of the new client
/// `user_name` - the username of the new client
/// `real_name` - the realname of the new client
/// `channels` - a list of channels to connect to
pub struct IrcConfig<'icfg> {
  pub host      : &'icfg str,
  pub port      : u16,

  pub nick_name : &'icfg str,
  pub user_name : &'icfg str,
  pub real_name : &'icfg str,
  
  pub channels  : Vec<&'icfg str>,
}