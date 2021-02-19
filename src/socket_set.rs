use zmq;
use std::sync::Arc;

pub struct SocketSet {
    pub pull_socket: Arc<zmq::Socket>,
    pub pub_socket: Arc<zmq::Socket>,
    pub sub_socket: Arc<zmq::Socket>,
    pub push_socket: Arc<zmq::Socket>,
}

impl SocketSet {
    pub fn new(ctx: &zmq::Context,
                   pull_url: &str,
                   pub_url: &str,
                   rhwm: i32,
                   shwm: i32)
                   -> SocketSet {
        let pull_sock = Arc::new(ctx.socket(zmq::PULL).unwrap());
        pull_sock.bind(pull_url).unwrap();
        let _res = pull_sock.set_rcvhwm(rhwm);

        let pub_sock = Arc::new(ctx.socket(zmq::PUB).unwrap());
        pub_sock.bind(pub_url).unwrap();
        let _res = pub_sock.set_sndhwm(shwm);

        let push_sock = Arc::new(ctx.socket(zmq::PUSH).unwrap());
        push_sock.connect(pull_url).unwrap();
        let _res = push_sock.set_rcvhwm(rhwm);

        let sub_sock = Arc::new(ctx.socket(zmq::SUB).unwrap());
        sub_sock.connect(pub_url).unwrap();
        let _res = sub_sock.set_rcvhwm(rhwm);

        SocketSet {
            pull_socket: pull_sock.clone(),
            pub_socket: pub_sock.clone(),
            sub_socket: sub_sock.clone(),
            push_socket: push_sock.clone(),
        }
    }
}