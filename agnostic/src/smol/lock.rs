#[derive(Debug)]
pub struct SmolMutex<T: ?Sized>(smol::lock::Mutex<T>);

impl<T: ?Sized> core::ops::Deref for SmolMutex<T> {
  type Target = smol::lock::Mutex<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: ?Sized> core::ops::DerefMut for SmolMutex<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug)]
pub struct SmolRwLock<T: ?Sized>(smol::lock::RwLock<T>);

impl<T: ?Sized> core::ops::Deref for SmolRwLock<T> {
  type Target = smol::lock::RwLock<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: ?Sized> core::ops::DerefMut for SmolRwLock<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[async_trait::async_trait]
impl<T: ?Sized> crate::lock::Mutex<T> for SmolMutex<T> {
  type Guard<'a> = smol::lock::MutexGuard<'a, T> where T: 'a;

  fn new(val: T) -> Self
  where
    T: Sized,
  {
    SmolMutex(smol::lock::Mutex::new(val))
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

#[async_trait::async_trait]
impl<T: ?Sized> crate::lock::RwLock<T> for SmolRwLock<T> {
  type ReadGuard<'a> = smol::lock::RwLockReadGuard<'a, T> where T: 'a;
  type WriteGuard<'a> = smol::lock::RwLockWriteGuard<'a, T> where T: 'a;

  fn new(val: T) -> Self
  where
    T: Sized,
  {
    SmolRwLock(smol::lock::RwLock::new(val))
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
