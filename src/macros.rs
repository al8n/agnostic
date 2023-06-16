#[cfg(feature = "net")]
macro_rules! cfg_async_std {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "async-std")]
      #[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
      $item
    )*
  }
}

#[cfg(feature = "net")]
macro_rules! cfg_tokio {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "tokio")]
      #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
      $item
    )*
  }
}

#[cfg(feature = "net")]
macro_rules! cfg_smol {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "smol")]
      #[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
      $item
    )*
  }
}

#[cfg(feature = "net")]
macro_rules! cfg_monoio {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "monoio")]
      #[cfg_attr(docsrs, doc(cfg(feature = "monoio")))]
      $item
    )*
  }
}
