pub struct Cluster{
  pub slots:Vec<Slots>,
  pub nodes:Vec<String>,
  pub redirections:Vec<Redirection>,

}
  
pub struct Slots{
  pub masters:Vec<String>,
  pub slaves:Vec<String>
}

enum RedisCmd{
  Get,
  Set,
  Del,
  Ping
}
pub struct Redirection{
  pub target: Redirect,
  pub cmd: RedisCmd
}
impl Redirection{
  pub fn new(target:Redirect,cmd:RedisCmd)->Self{
    Redirection{
      target,
      cmd
    }
  }
  
}
pub enum Redirect{
  Move{
    slot:usize,
    to:String
  },
  //Ask for a move
  Ask{
    slot:usize,
    to:String
  }
}
impl Redirect{
  pub fn new(is_move:bool,slot:usize,to:String)->Self{
    if is_move{
      Redirect::Move{
        slot,
        to
      }
    }else{
      Redirect::Ask{
        slot,
        to
      }
    }
  }
}
//crc16
const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);

impl Slots{
  pub fn new()->Self{
    Slots{
      masters:Vec::new(),
      slaves:Vec::new()
    }
  }
  pub fn add_master(&mut self,addr:String){
    self.masters.push(addr);
  }
  pub fn add_slave(&mut self,addr:String){
    self.slaves.push(addr);
  }
  pub fn get_master(&self)->String{
    self.masters[0].clone()
  }
  pub fn get_slave(&self)->String{
    self.slaves[0].clone()
  }

}
