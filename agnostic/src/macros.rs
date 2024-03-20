/// Macro to conditionally compile items for `async-std` feature
#[cfg(feature = "async-std")]
#[macro_export]
macro_rules! cfg_async_std {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "async-std")]
      #[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
      $item
    )*
  }
}

/// Macro to conditionally compile items for `tokio` feature
#[cfg(feature = "tokio")]
#[macro_export]
macro_rules! cfg_tokio {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "tokio")]
      #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
      $item
    )*
  }
}

/// Macro to conditionally compile items for `smol` feature
#[cfg(feature = "smol")]
#[macro_export]
macro_rules! cfg_smol {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "smol")]
      #[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
      $item
    )*
  }
}
