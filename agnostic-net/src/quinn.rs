use agnostic_lite::RuntimeLite;


/// The Quinn abstraction for this runtime
pub trait Quinn {
  /// The runtime type
  type Runtime: RuntimeLite;

  /// Abstract implementation of a UDP socket for runtime independence
  type UdpSocket: quinn::AsyncUdpSocket;
}
