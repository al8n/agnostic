#[derive(Debug)]
pub struct TokioWasmMutex<T: ?Sized>(tokio::sync::Mutex<T>);

impl<T: ?Sized> core::ops::Deref for TokioWasmMutex<T> {
  type Target = tokio::sync::Mutex<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: ?Sized> core::ops::DerefMut for TokioWasmMutex<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[async_trait::async_trait]
impl<T: ?Sized> crate::lock::Mutex<T> for TokioWasmMutex<T> {
  type Guard<'a> = tokio::sync::MutexGuard<'a, T> where T: 'a;

  fn new(val: T) -> Self
  where
    T: Sized,
  {
    TokioWasmMutex(tokio::sync::Mutex::new(val))
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
    self.0.try_lock().ok()
  }
}

#[derive(Debug)]
pub struct TokioWasmRwLock<T: ?Sized>(tokio::sync::RwLock<T>);

impl<T: ?Sized> core::ops::Deref for TokioWasmRwLock<T> {
  type Target = tokio::sync::RwLock<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: ?Sized> core::ops::DerefMut for TokioWasmRwLock<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[async_trait::async_trait]
impl<T: ?Sized> crate::lock::RwLock<T> for TokioWasmRwLock<T> {
  type ReadGuard<'a> = tokio::sync::RwLockReadGuard<'a, T> where T: 'a;
  type WriteGuard<'a> = tokio::sync::RwLockWriteGuard<'a, T> where T: 'a;

  fn new(val: T) -> Self
  where
    T: Sized,
  {
    TokioWasmRwLock(tokio::sync::RwLock::new(val))
  }

  async fn read<'a>(&'a self) -> Self::ReadGuard<'a>
  where
    T: Send + Sync,
  {
    self.0.read().await
  }

  async fn write<'a>(&'a self) -> Self::WriteGuard<'a>
  where
    T: Send + Sync,
  {
    self.0.write().await
  }

  fn try_read(&self) -> Option<Self::ReadGuard<'_>>
  where
    T: Send + Sync,
  {
    self.0.try_read().ok()
  }

  fn try_write(&self) -> Option<Self::WriteGuard<'_>>
  where
    T: Send + Sync,
  {
    self.0.try_write().ok()
  }
}
