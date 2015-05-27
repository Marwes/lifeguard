#![allow(dead_code)]
use std::cell::RefCell;
use std::fmt;
use std::ops::{Drop, Deref, DerefMut};
use std::convert::{AsRef, AsMut};

pub trait Recycleable {
  fn new() -> Self;
  fn reset(&mut self);
}

pub trait InitializeWith<T> {
  fn initialize_with(&mut self, source: T);
}

impl Recycleable for String {
  #[inline] 
  fn new() -> String {
    String::new()
  }
  #[inline] 
  fn reset(&mut self) {
    self.clear();
  }
}

impl <T> Recycleable for Vec<T> {
  #[inline] 
  fn new() -> Vec<T> {
    Vec::new()
  }
  #[inline] 
  fn reset(&mut self) {
    self.clear();
  }
}

impl <A> InitializeWith<A> for String where A : AsRef<str> {
  #[inline] 
  fn initialize_with(&mut self, source: A) {
    let s : &str = source.as_ref();
    self.push_str(s);
  }
}

pub struct Recycled<'pool, T> where T : Recycleable + 'pool {
  pub value: Option<T>,
  pool: &'pool RefCell<Vec<T>>
}

impl <'pool, T> Drop for Recycled<'pool, T> where T : Recycleable {
  #[inline] 
  fn drop(&mut self) {
    if let Some(mut value) = self.value.take() {
      value.reset();
      self.pool.borrow_mut().push(value);
    }
  }
}

impl <'pool, T> AsRef<T> for Recycled<'pool, T> where T : Recycleable {
   fn as_ref(&self) -> &T {
    match self.value.as_ref() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <'pool, T> AsMut<T> for Recycled<'pool, T> where T : Recycleable {
   fn as_mut(&mut self) -> &mut T {
    match self.value.as_mut() {
      Some(v) => v,
      None => panic!("Recycled<T> smartpointer missing its value.")
    }
  }
}

impl <'pool, T> fmt::Debug for Recycled<'pool, T> where T : fmt::Debug + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
    }
  }
}

impl <'pool, T> fmt::Display for Recycled<'pool, T> where T : fmt::Display + Recycleable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.value {
      Some(ref s) => s.fmt(f),
      None => write!(f, "Empty Recycled<T>")
    }
  }
}

impl <'pool, T> Deref for Recycled<'pool, T> where T : Recycleable{
  type Target = T;
  #[inline] 
  fn deref<'a>(&'a self) -> &'a T {
    self.as_ref()
  }
}

impl <'pool, T> DerefMut for Recycled<'pool, T> where T : Recycleable {
  #[inline] 
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    self.as_mut()
  }
}

impl <'pool, T> Recycled<'pool, T> where T : Recycleable {
  #[inline] 
  pub fn new (pool: &'pool RefCell<Vec<T>>, value: T) -> Recycled<'pool, T> {
    Recycled {
      value: Some(value),
      pool: pool
    }
  }
  
  #[inline] 
  pub fn new_from<A>(pool: &'pool RefCell<Vec<T>>, mut value: T, source: A) -> Recycled<'pool, T> where T : InitializeWith<A> {
    value.initialize_with(source);
    Recycled {
      value: Some(value),
      pool: pool
    }
  }

  #[inline] 
  pub fn detach(mut self) -> T {
    let value = self.value.take().unwrap();
    drop(self);
    value
  }
}

pub struct Pool <T> where T : Recycleable {
  values: RefCell<Vec<T>>,
}

impl <T> Pool <T> where T: Recycleable {
  #[inline]
  pub fn with_size(size: u32) -> Pool <T> {
    let values: Vec<T> = 
      (0..size)
      .map(|_| T::new() )
      .collect();
    Pool {
      values: RefCell::new(values),
    }
  }

  #[inline] 
  pub fn attach<'pool> (&'pool self, value: T) -> Recycled<'pool, T> {
    let pool_reference = &self.values;
    Recycled::new(pool_reference, value)
  }

  #[inline] 
  pub fn detached(&self) -> T {
    match self.values.borrow_mut().pop() {
      Some(v) => v,
      None => T::new()
    }
  }

  #[inline] 
  pub fn new<'pool>(&'pool self) -> Recycled<'pool, T> {
    let t = self.detached();
    let pool_reference = &self.values;
    Recycled::new(pool_reference, t)
  }
 
  #[inline(always)] 
  pub fn new_from<'pool, A>(&'pool self, source: A) -> Recycled<'pool, T> where T: InitializeWith<A> {
    let t = self.detached();
    let pool_reference = &self.values;
    Recycled::new_from(pool_reference, t, source)
  }

  #[inline] 
  pub fn size(&self) -> usize {
    self.values.borrow().len()
  }
}
