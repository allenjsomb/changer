use zmq;

#[derive(Debug, Clone)]
pub struct ChangerSocket {
    pub pull_socket: zmq::Socket,
    pub pub_socket: zmq::Socket,
    pub sub_socket: zmq::Socket,
    pub push_socket: zmq::Socket
}

impl ChangerSocket {
    pub fn new(ctx: &zmq::Context,
               url: &str,
               stype: zmq::SocketType,
               bind: bool,
               rhwm: i32,
               shwm: i32)
               -> Option<ChangerSocket> {
        let sock = ctx.socket(stype);
        if sock.is_err() {
            return None;
        }

        None
    }
}

impl Drop for ChangerSocket {
    fn drop(&mut self) {
        self.socket.
    }
}