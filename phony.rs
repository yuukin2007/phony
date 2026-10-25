use std::{fs,fmt,io,vec};
use std::collections::HashMap;


pub struct AndOption<A,B>{a:A, b:Option<B>}

impl<A,B> AndOption<A,B>{
 pub fn new(in_a:A) -> AndOption<A,B>{
  AndOption{a:in_a,b:None}
 }
 pub fn clear(&mut self){
  self.b = None;
 }
 pub fn set(&mut self, in_b:B){
  self.b = Some(in_b);
 }
 pub fn get_a(&self) -> &A{
  &self.a
 }
 pub fn get_b(&self) -> &Option<B>{
  &self.b
 }
}

impl<A:Clone,B:Clone> Clone for AndOption<A,B>{
 fn clone(&self) -> Self{
  AndOption{a:self.a.clone(), b:self.b.clone()}
 }
}


//<K:name, V:(cmds, stack)>
pub struct VM{
 fns: HashMap< Vec<u8>, AndOption<Vec<Vec<Vec<u8>>>, Vec<Vec<u8>>> >
}

impl VM{
 pub fn new() -> Self{
  VM{fns:HashMap::new()}
 }

 pub fn exec(&mut self, cmd:Vec<Vec<u8>>) -> Vec<u8>{
  let len = cmd.len();
  let mut iter = cmd.into_iter();
  let name = match iter.next(){
   Some(v) => v,
   None => return Vec::new()
  };
  if name[0] == b'('{error(Internal, "function name include '('");}
  match &*name{

   b"teigi" => {
    match len{
     1 => {error(Internal, "it can't define, because missing identifier");}
     2 => {error(Internal, "it can't define, because the function nonsence");}
     _ => {}
    }
    self.def(iter.next().unwrap(), iter);
   }

   b"hikisu" => {
    match len{
     1 => error(Internal, "missing index specifier of hikisu"),
     2 => error(Internal, "missing function specifier of hikisu"),
     _ => {}
    };
    let mut n_vec = iter.next().unwrap();
    if n_vec[0] == b'('{n_vec = self.exec(split(&mut n_vec.into_iter()).unwrap());}
    let n = n_vec.into_iter()
      .map(|x| match x{
       b'0' => 0, b'1' => 1, 
       c => eformat!(Internal, "hikisu: unexpected char'0x{:x}'", c)
      })
      .reduce(|x,y| (x<<1)+y).unwrap();
    let f = iter.next().unwrap();
    return match self.fns.get_mut(&f){
     Some(a) => match a.get_b(){
      Some(v) => v[n].clone(),
      None => eformat!(Internal, "hikisu: not executing function of '{}'", from_utf8(&*f))
     },
     None => eformat!(Internal, "hikisu: not exists function of '{}'", from_utf8(&*f))
    };
   }

   b"yunyu" => {
    match len{
     1 => {error(Internal, "yunyu: missing mode specifier");}
     2 => {error(Internal, "yunyu: missing file specifier");}
     _ => {}
    }
    match &*iter.next().unwrap(){
     b"kansu" => {self.import_fn(iter.next().unwrap());}
     b"ko-do" => {self.import_code(iter.next().unwrap());}
     m => {eformat!(Internal, "yunyu: non-existent mode'{}'", from_utf8(m))}
    }
   }


   b"|" => {
    let split_args = |x:Vec<u8>|{
     if x[0] == b'('{
      split_all(&mut self.exec(split(&mut x.into_iter()).unwrap()).into_iter())
     }
     else{vec![x]}
    };
    let result = iter.flat_map(split_args).collect();
    return self.exec(result);
   }

   &[0xf0,0x9f,0xa4,0x94] => {
    match len{
     1 => {error(Internal, "🤔: missing bool value");}
     2 => {error(Internal, "🤔: missing command");}
     _ => {}
    }
    let mut bool_val_vec = iter.next().unwrap();
    if bool_val_vec[0] == b'(' {
     bool_val_vec = self.exec(split(&mut bool_val_vec.into_iter()).unwrap());
    }
    let bool_val = match &*bool_val_vec{
     b"0" => true, b"1" => false,
     b => eformat!(Internal, "🤔: unexpected string'{}'", from_utf8(b))
    };
    if bool_val{
     return self.exec(split(&mut iter.next().unwrap().into_iter()).unwrap());
    }
   }

   b"hunsai" => {
    
   }

   b"gigante" => {
    match iter.next(){
     Some(v) => eformat!(Exception, "{}", from_utf8(&*v)),
     None => eformat!(Exception, "[empty]")
    }
   }

   f => {
    return self.exec_fn(f.to_vec(), iter);
   }

  }
  Vec::new()
 }

 pub fn exec_fn(&mut self, name:Vec<u8>, iter:vec::IntoIter<Vec<u8>>) -> Vec<u8>{
  match self.fns.get_mut(&name){
   Some(a) => {
    a.set(iter.as_slice().to_vec());
    let result = a.get_a().clone().into_iter()
     .map(|x| self.exec(x))
     .reduce(|mut x,mut y| {x.append(&mut y);x}).unwrap();
    self.fns.get_mut(&name).unwrap().clear();
    result
   },
   None => eformat!(Internal, "not exists function of '{}'", from_utf8(&*name))
  }
 }

 pub fn def(&mut self, name:Vec<u8>, iter:vec::IntoIter<Vec<u8>>){
  if self.fns.contains_key(&name){eformat!(Internal, "'{}' is already defained", from_utf8(&*name))};
  self.fns.insert(
   name,
   AndOption::new(iter
    .map(|x| split(&mut x.into_iter()).unwrap())
    .collect()) );
 }

 pub fn import_fn(&mut self, name:Vec<u8>){
  let file = match fs::File::open(from_utf8(&*name.clone())) {
   Ok(f) => f,
   Err(e) => error(IO, e)
  };
  let mut bytes = Bytes::new(file);
  let mut cmds = Vec::new();
  while let Some(c) = split(&mut bytes){cmds.push(c);}
  if self.fns.contains_key(&name){eformat!(Internal, "'{}' is already defained", from_utf8(&*name))};
  self.fns.insert(name,AndOption::new(cmds));
 }

 pub fn import_code(&mut self, name:Vec<u8>) -> Vec<u8>{
  let file = match fs::File::open(from_utf8(&*name.clone())) {
   Ok(f) => f,
   Err(e) => error(IO, e)
  };
  let mut bytes = Bytes::new(file);
  let mut result = Vec::new();
  while let Some(c) = split(&mut bytes){result.append(&mut self.exec(c))};
  result
 }
}

impl Clone for VM{
 fn clone(&self) -> Self{
  VM{fns:self.fns.clone()}
 }
}

//"(a  (b c) d) (...)" -> ["a", "(b c)", "d"]
pub fn split<I>(target:&mut I) -> Option<Vec<Vec<u8>>>
where I:Iterator<Item = u8>{
 let mut buf;
 macro_rules! next_or{
   ($e:expr) => {match target.next(){Some(b) => {buf = b} None => {$e}}}
 }
 macro_rules! skip_space {($e:expr) =>{
  loop{
   next_or!{$e}
   if buf != b' '
   && buf != b'\n'{break}
  }
 }}
 skip_space!{return None};
 if buf != b'('{
  eformat!(Parse, "the depth of '0x{:x}' is 0", buf);
 }
 let mut vec = vec![Vec::new()];
 let mut dep = 1;
 skip_space!{error(Parse, "moved to eof during parse")};
 loop{
  if buf == b'('{
   dep+=1;
  }else
  if buf == b')'{
   dep-=1;
  }else
  if buf == b'\\'{
   if dep != 1{
    let len = vec.len();
    vec[len-1].push(buf);
   }
   next_or!{error(Parse, "moved to eof during parse")};
  }else
  if dep == 1
  && buf == b' '
  || buf == b'\n'{
   skip_space!{error(Parse, "moved to eof during parse")};
   vec.push(Vec::new());
   continue;
  }
  if dep == 0{break}
  let len = vec.len();
  vec[len-1].push(buf);
  next_or!{error(Parse, "moved to eof during parse")};
 }
 Some(vec)
}

pub fn split_all<I>(target:&mut I) -> Vec<Vec<u8>>
where I:Iterator<Item = u8>{
 let mut buf;
 macro_rules! next_or{
   ($e:expr) => {match target.next(){Some(b) => {buf = b} None => {$e}}}
 }
 macro_rules! skip_space {($e:expr) =>{
  loop{
   next_or!{$e}
   if buf != b' '
   && buf != b'\n'{break}
  }
 }}
 let mut vec = vec![Vec::new()];
 skip_space!{return Vec::new()};
 let mut dep = 1;
 loop{
  if buf == b'('{
   dep+=1;
  }else
  if buf == b')'{
   dep-=1;
  }else
  if buf == b'\\'{
   if dep != 1{
    let len = vec.len();
    vec[len-1].push(buf);
   }
   next_or!{error(Parse, "moved to eof during parse")};
  }else
  if dep == 1
  && buf == b' '
  || buf == b'\n'{
   skip_space!{error(Parse, "moved to eof during parse")};
   vec.push(Vec::new());
   continue;
  }
  if dep == 0{error(Parse, "depth reached 0")}
  let len = vec.len();
  vec[len-1].push(buf);
  next_or!{break};
 }
 vec
}

#[derive(Debug)]
pub enum Kind{
 IO,
 Internal,
 Exception,
 Parse
}

use self::Kind::*;

impl fmt::Display for Kind{
 fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result{
  fmt::Debug::fmt(self, f)
 }
}

pub fn error<D:fmt::Display>(kind:Kind, detail:D) -> !{
 eprintln!("ERROR\n kind: {}\n detail: {}", kind, detail);
 std::process::exit(1)
}

#[macro_export]
macro_rules! eformat{
 ($kind:ident, $($arg:tt)*) => {error($kind, format!($($arg)*))}
}

//Uncomment when compiling test.rs
//use ::eformat;

fn from_utf8(slice:&[u8]) -> &str{
 match std::str::from_utf8(slice){
  Ok(s) => s,
  Err(e) => eformat!(Parse, "Invalid UTF-8 sequence: {}", e)
 }
}

//if throwing IOerror, do error()
pub struct Bytes<R:io::Read> {inner:R}

impl<R:io::Read> Iterator for Bytes<R>{
 type Item = u8;

 fn next(&mut self) -> Option<u8>{
  let mut byte = 0;
  loop{
   return match self.inner.read(std::slice::from_mut(&mut byte)){
    Ok(0) => None,
    Ok(_) => Some(byte),
    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
    Err(e) => error(IO, e)
   };
  }
 }
}

impl<R:io::Read> Bytes<R>{
 fn new(inner:R) -> Self{
  Bytes{inner:inner}
 }
}








#[allow(dead_code)]
fn main(){
 let mut args = std::env::args();
 let mut vm = VM::new();
 args.next();
 let name =  match args.next(){
  Some(s) => s,
  None => error(IO, "missing file specifier")
 }.into_bytes();
 vm.import_code(b"/Users/yuhkinumata/.phony/lib.ph".to_vec());
 vm.import_fn(name.clone());
 println!("{}", from_utf8(&*vm.exec_fn(name, args.map(|x| x.into_bytes())
  .collect::<Vec<Vec<u8>>>().into_iter())));
}