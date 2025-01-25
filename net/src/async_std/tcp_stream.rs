tcp_stream!("async-std" {
  {
    impl TryFrom<std::net::TcpStream> for TcpStream {
      type Error = ::std::io::Error;
  
      #[inline]
      fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
        Ok(Self::from(::async_std::net::TcpStream::from(stream)))
      }
    }

    impl_as_raw_fd!(TcpStream.stream);
    impl_as_fd_async_std!(TcpStream.stream);
  }
});
