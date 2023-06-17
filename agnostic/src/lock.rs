pub trait Lock {
  type Mutex<T>;
  type RwLock;
}

#[async_trait::async_trait]
pub trait Mutex<T: ?Sized> {
  type Guard<'a>
  where
    T: 'a,
    Self: 'a;

  fn new(val: T) -> Self
  where
    T: Sized;

  async fn lock<'a>(&'a self) -> Self::Guard<'a>
  where
    T: Send;

  fn try_lock(&self) -> Option<Self::Guard<'_>>
  where
    T: Send;
}

#[async_trait::async_trait]
pub trait RwLock<T: ?Sized> {
  type ReadGuard<'a>
  where
    T: 'a,
    Self: 'a;
  type WriteGuard<'a>
  where
    T: 'a,
    Self: 'a;

  fn new(val: T) -> Self
  where
    T: Sized;

  async fn read<'a>(&'a self) -> Self::ReadGuard<'a>
  where
    T: Send + Sync;

  fn try_read(&self) -> Option<Self::ReadGuard<'_>>
  where
    T: Send + Sync;

  async fn write<'a>(&'a self) -> Self::WriteGuard<'a>
  where
    T: Send + Sync;

  fn try_write(&self) -> Option<Self::WriteGuard<'_>>
  where
    T: Send + Sync;
}
