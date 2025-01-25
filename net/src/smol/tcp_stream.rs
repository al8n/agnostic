tcp_stream!("smol" {
  {
    impl TryFrom<std::net::TcpStream> for TcpStream {
      type Error = ::std::io::Error;
  
      #[inline]
      fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
        ::smol::net::TcpStream::try_from(stream).map(Self::from)
      }
    }
  
    impl_as!(TcpStream.stream);
  }
});
