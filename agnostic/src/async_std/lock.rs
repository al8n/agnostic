#[derive(Debug)]
pub struct AsyncStdMutex<T: ?Sized>(async_std::sync::Mutex<T>);

impl<T: ?Sized> core::ops::Deref for AsyncStdMutex<T> {
  type Target = async_std::sync::Mutex<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: ?Sized> core::ops::DerefMut for AsyncStdMutex<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[async_trait::async_trait]
impl<T: ?Sized> crate::lock::Mutex<T> for AsyncStdMutex<T> {
  type Guard<'a> = async_std::sync::MutexGuard<'a, T> where T: 'a;

  fn new(val: T) -> Self
  where
    T: Sized,
  {
    AsyncStdMutex(async_std::sync::Mutex::new(val))
  }

  async fn lock<'a>(&'a self) -> Self::Guard<'a>
  where
    T: Send,
  {
    self.0.lock().await
  }

  fn try_lock(&self) -> Option<Self::Guard<'_>>
  where
    T: Send,
  {
    self.0.try_lock()
  }
}

#[derive(Debug)]
pub struct AsyncStdRwLock<T: ?Sized>(async_std::sync::RwLock<T>);

impl<T: ?Sized> core::ops::Deref for AsyncStdRwLock<T> {
  type Target = async_std::sync::RwLock<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: ?Sized> core::ops::DerefMut for AsyncStdRwLock<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[async_trait::async_trait]
impl<T: ?Sized> crate::lock::RwLock<T> for AsyncStdRwLock<T> {
  type ReadGuard<'a> = async_std::sync::RwLockReadGuard<'a, T> where T: 'a;
  type WriteGuard<'a> = async_std::sync::RwLockWriteGuard<'a, T> where T: 'a;

  fn new(val: T) -> Self
  where
    T: Sized,
  {
    AsyncStdRwLock(async_std::sync::RwLock::new(val))
  }

  async fn read<'a>(&'a self) -> Self::ReadGuard<'a>
  where
    T: Send + Sync,
  {
    self.0.read().await
  }

  fn try_read(&self) -> Option<Self::ReadGuard<'_>>
  where
    T: Send + Sync,
  {
    self.0.try_read()
  }

  async fn write<'a>(&'a self) -> Self::WriteGuard<'a>
  where
    T: Send + Sync,
  {
    self.0.write().await
  }

  fn try_write(&self) -> Option<Self::WriteGuard<'_>>
  where
    T: Send + Sync,
  {
    self.0.try_write()
  }
}
