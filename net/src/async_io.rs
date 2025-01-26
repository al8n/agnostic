macro_rules! owned_halves {
  ($runtime:literal) => {
    paste::paste! {
      #[doc = "The [`OwnedReadHalf`](crate::OwnedReadHalf) implementation for [`" $runtime "`] runtime"]
      ///
      #[doc = "[`" $runtime "`]: https://docs.rs/" $runtime]
      #[derive(Debug)]
      pub struct OwnedReadHalf {
        pub(super) stream: ::[< $runtime:snake >]::net::TcpStream,
      }

      #[doc = "The [`OwnedWriteHalf`](crate::OwnedWriteHalf) implementation for [`" $runtime "`] runtime"]
      ///
      #[doc = "[`" $runtime "`]: https://docs.rs/" $runtime]
      #[derive(Debug)]
      pub struct OwnedWriteHalf {
        pub(super) stream: ::[< $runtime:snake >]::net::TcpStream,
        pub(super) shutdown_on_drop: bool,
      }

      impl Drop for OwnedWriteHalf {
        fn drop(&mut self) {
          if self.shutdown_on_drop {
            let _ = self.stream.shutdown(::std::net::Shutdown::Write);
          }
        }
      }

      impl ::futures_util::AsyncRead for OwnedReadHalf {
        fn poll_read(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &mut [u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
        }
      }

      impl ::futures_util::AsyncWrite for OwnedWriteHalf {
        fn poll_write(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &[u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_write(cx, buf)
        }

        fn poll_flush(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_flush(cx)
        }

        fn poll_close(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_close(cx)
        }
      }

      #[cfg(feature = "tokio-io")]
      impl ::tokio::io::AsyncRead for OwnedReadHalf {
        fn poll_read(
          self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &mut ::tokio::io::ReadBuf<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut ::agnostic_io::tokio_compat::FuturesAsyncReadCompatExt::compat(
            self.get_mut(),
          ))
          .poll_read(cx, buf)
        }
      }

      #[cfg(feature = "tokio-io")]
      impl ::tokio::io::AsyncWrite for OwnedWriteHalf {
        fn poll_write(
          self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &[u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
          ::std::pin::Pin::new(&mut ::agnostic_io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
            .poll_write(cx, buf)
        }

        fn poll_flush(self: ::std::pin::Pin<&mut Self>, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut ::agnostic_io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
            .poll_flush(cx)
        }

        fn poll_shutdown(self: ::std::pin::Pin<&mut Self>, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut ::agnostic_io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
            .poll_shutdown(cx)
        }
      }

      impl $crate::OwnedReadHalf for OwnedReadHalf {
        type Runtime = ::agnostic_lite::[< $runtime:snake >]::[< $runtime:camel Runtime >];

        tcp_stream_owned_read_half_common_methods!(stream);
      }

      impl $crate::OwnedWriteHalf for OwnedWriteHalf {
        type Runtime = ::agnostic_lite::[< $runtime:snake >]::[< $runtime:camel Runtime >];

        fn forget(mut self) {
          self.shutdown_on_drop = false;
          drop(self);
        }

        tcp_stream_owned_write_half_common_methods!(stream);
      }
    }
  };
}

macro_rules! tcp_stream {
  ($runtime:literal { $converter: expr }) => {
    owned_halves!($runtime);

    paste::paste! {
      /// Error indicating that two halves were not from the same socket, and thus could
      /// not be reunited.
      #[derive(Debug)]
      pub struct ReuniteError(
        pub OwnedReadHalf,
        pub OwnedWriteHalf,
      );

      impl core::fmt::Display for ReuniteError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          write!(
            f,
            "tried to reunite halves that are not from the same socket"
          )
        }
      }

      impl core::error::Error for ReuniteError {}

      #[doc = "The [`TcpStream`](crate::TcpStream) implementation for [`" $runtime "`] runtime"]
      ///
      #[doc = "[`" $runtime "`]: https://docs.rs/" $runtime]
      #[derive(Debug)]
      #[repr(transparent)]
      pub struct TcpStream {
        stream: ::[< $runtime:snake >]::net::TcpStream,
      }

      impl From<::[< $runtime:snake >]::net::TcpStream> for TcpStream {
        fn from(stream: ::[< $runtime:snake >]::net::TcpStream) -> Self {
          Self { stream }
        }
      }

      const _: () = $converter;

      impl TryFrom<socket2::Socket> for TcpStream {
        type Error = ::std::io::Error;

        fn try_from(socket: socket2::Socket) -> ::std::io::Result<Self> {
          Self::try_from(::std::net::TcpStream::from(socket))
        }
      }

      impl ::futures_util::AsyncRead for TcpStream {
        fn poll_read(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &mut [u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
        }
      }

      impl ::futures_util::AsyncWrite for TcpStream {
        fn poll_write(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &[u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_write(cx, buf)
        }

        fn poll_flush(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut std::task::Context<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_flush(cx)
        }

        fn poll_close(
          mut self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut (&mut self.stream)).poll_close(cx)
        }
      }

      #[cfg(feature = "tokio-io")]
      impl tokio::io::AsyncRead for TcpStream {
        fn poll_read(
          self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &mut ::tokio::io::ReadBuf<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut agnostic_io::tokio_compat::FuturesAsyncReadCompatExt::compat(
            self.get_mut(),
          ))
          .poll_read(cx, buf)
        }
      }

      #[cfg(feature = "tokio-io")]
      impl ::tokio::io::AsyncWrite for TcpStream {
        fn poll_write(
          self: ::std::pin::Pin<&mut Self>,
          cx: &mut ::std::task::Context<'_>,
          buf: &[u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
          ::std::pin::Pin::new(&mut agnostic_io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
            .poll_write(cx, buf)
        }

        fn poll_flush(self: ::std::pin::Pin<&mut Self>, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut agnostic_io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
            .poll_flush(cx)
        }

        fn poll_shutdown(self: ::std::pin::Pin<&mut Self>, cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<::std::io::Result<()>> {
          ::std::pin::Pin::new(&mut agnostic_io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
            .poll_shutdown(cx)
        }
      }

      impl $crate::TcpStream for TcpStream {
        type Runtime = ::agnostic_lite::[< $runtime:snake >]::[< $runtime:camel Runtime >];
        type OwnedReadHalf = OwnedReadHalf;
        type OwnedWriteHalf = OwnedWriteHalf;
        type ReuniteError = ReuniteError;

        tcp_stream_common_methods!($runtime::stream);

        fn shutdown(&self, how: ::std::net::Shutdown) -> ::std::io::Result<()> {
          self.stream.shutdown(how)
        }

        fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
          (
            Self::OwnedReadHalf {
              stream: self.stream.clone(),
            },
            Self::OwnedWriteHalf {
              stream: self.stream,
              shutdown_on_drop: true,
            },
          )
        }

        fn reunite(
          read: Self::OwnedReadHalf,
          mut write: Self::OwnedWriteHalf,
        ) -> ::core::result::Result<Self, Self::ReuniteError>
        where
          Self: Sized,
        {
          macro_rules! reunite_error {
            () => {
              ::core::result::Result::Err(ReuniteError(read, write))
            };
          }

          match (read.stream.local_addr(), write.stream.local_addr()) {
            (Ok(local_addr_read), Ok(local_addr_write)) if local_addr_read == local_addr_write => {}
            _ => return reunite_error!(),
          }

          match (read.stream.peer_addr(), write.stream.peer_addr()) {
            (Ok(peer_addr_read), Ok(peer_addr_write)) if peer_addr_read == peer_addr_write => {}
            _ => return reunite_error!(),
          }

          write.shutdown_on_drop = false;
          ::core::result::Result::Ok(Self {
            stream: read.stream,
          })
        }
      }
    }
  };
}
