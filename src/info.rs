use std::collections;

use message;
use utils::debug;

pub struct IrcInfo {
  pub nick_name : String,
  pub user_name : String,
  pub real_name : String,
  
  pub channels  : Vec < String >,
  names         : collections::HashMap < String, Vec < String > >,
  prep_names    : Vec < String >,
}

impl Clone for IrcInfo {
  fn clone( &self ) -> IrcInfo {
    IrcInfo {
      nick_name : self.nick_name.clone( ),
      user_name : self.user_name.clone( ),
      real_name : self.real_name.clone( ),
      channels  : self.channels.clone( ),
      names     : self.names.clone( ),
      prep_names: Vec::new( ),
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
      names     : collections::HashMap::new( ),
      prep_names: Vec::new( ),
    }
  }
  
  pub fn update_info( &mut self, msg : message::Message ) {
    match msg.code.as_slice( ) {
      // update nickname on NICK message
      "NICK" => { 
        if msg.nick( ).unwrap_or( String::from_str( "" ) ) == self.nick_name {
          self.nick_name = msg.param( 1 ).unwrap( ).to_string( );
        }
      },
      // add channels on JOIN message
      "JOIN" => {
        if msg.nick( ).unwrap_or( String::from_str( "" ) ) == self.nick_name {
          self.channels.push( msg.param( 1 ).unwrap( ).to_string( ) );
        } else {
          self.add_to_channel( msg.param( 1 ).unwrap( ).to_string( ), msg.nick( ).unwrap_or( String::from_str( "" ) ) );
        }
      },
      // remove channels on PART message
      "PART" => {
        if msg.nick( ).unwrap_or( String::from_str( "" ) ) == self.nick_name {
          for i in range( 0, self.channels.len( ) ) {
            if self.channels[i] == msg.param( 1 ).unwrap( ) {
              self.drop_channel_names( msg.param( 1 ).unwrap( ).to_string( ) );
              self.channels.remove( i );
              break;
            }
          }
        } else {
          self.remove_from_channel( msg.param( 1 ).unwrap( ).to_string( ), msg.nick( ).unwrap_or( String::from_str( "" ) ) );
        }
      },
      // remove channels on channel errors
      "403" | "405" | "437" | "471" | "473" | "474" | "475" | "476" => {
        for i in range( 0, self.channels.len( ) ) {
          if self.channels[i] == msg.param( 1 ).unwrap( ) {
            self.drop_channel_names( msg.param( 1 ).unwrap( ).to_string( ) );
            self.channels.remove( i );
            break;
          }
        }
      },
      _   => (),
    }
  }
  
  pub fn prep_channel_names( &mut self, msg : message::Message ) {
    let mut name_list = match msg.trailing( ) {
      Some( trail ) => trail.as_slice( ).split_str( " " ),
      None          => return,
    };
    for name in name_list {
      self.prep_names.push( String::from_str( name ) );
    }
  }
  
  pub fn set_channel_names( &mut self, chan : String ) {
    // drop the name list if it already exists in our map
    if self.names.contains_key( &chan ) {
      self.drop_channel_names( chan.clone( ) );
    }
    
    // insert the name list into out map and get a pointer to it
    self.names.insert( chan.clone( ), Vec::new( ) );
    let chan_list = self.names.get_mut( &chan ).unwrap( );
    
    // populate the vector
    for name in self.prep_names.iter( ) {
      chan_list.push( name.clone( ) );
    }
    
    // clear our prep list
    self.prep_names.clear( );
  }
  
  pub fn get_channel_names( &self, chan : String ) -> Option< &Vec < String > > {
    self.names.get( &chan )
  }
  
  fn drop_channel_names( &mut self, chan : String ) {
    match self.names.get_mut( &chan ) {
      Some( list )  => list.clear( ),
      None          => {
        debug::warn( "drop name list", "name list does not exist" );
        return;
      },
    }
    self.names.remove( &chan );
  }
  
  fn add_to_channel( &mut self, chan : String, nick : String ) {
    match self.names.get_mut( &chan ) {
      Some( list )  => list.push( nick ),
      None          => debug::warn( "add nick to name list", "name list does not exist" ),
    }
  }
  
  fn remove_from_channel( &mut self, chan : String, nick : String ) {
    let chan_list = match self.names.get_mut( &chan ) {
      Some( list )  => list,
      None          => {
        debug::warn( "remove nick from name list", "name list does not exist" );
        return;
      },
    };
    for i in range( 0, chan_list.len( ) ) {
      if chan_list[i] == nick {
        chan_list.remove( i );
        break;
      }
    }
  }
}

impl Drop for IrcInfo {
  fn drop ( &mut self ) {
    for (_,list) in self.names.iter_mut( ) {
      list.clear( );
    }
    self.names.clear( );
    self.channels.clear( );
  }
}