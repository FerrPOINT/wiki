//! Owned PostgreSQL COMMIT fault fixture shared by the Wiki HTTP and storage regressions.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, UnixStream},
};
pub(super) struct FaultProxy {
    pub(super) port: u16,
    pub(super) lost: Arc<AtomicBool>,
    pub(super) armed: Arc<AtomicBool>,
    pub(super) refuse_readback: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for FaultProxy {
    fn drop(&mut self) {
        self.refuse_readback.store(true, Ordering::SeqCst);
        self.task.abort();
    }
}
impl FaultProxy {
    pub(super) async fn start(
        socket: String,
        refuse_readback: bool,
        drop_commit_failure: bool,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let lost = Arc::new(AtomicBool::new(false));
        let state = lost.clone();
        let armed = Arc::new(AtomicBool::new(false));
        let enabled = armed.clone();
        let refuse_readback = Arc::new(AtomicBool::new(refuse_readback));
        let refusal = refuse_readback.clone();
        let task = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                let accepted = tokio::select! {
                    accepted=listener.accept()=>Some(accepted),
                    result=connections.join_next(),if !connections.is_empty()=>{
                        result.unwrap().unwrap();
                        None
                    }
                };
                let Some(accepted) = accepted else {
                    continue;
                };
                let (client, _) = accepted.unwrap();
                let lost = state.clone();
                let socket = socket.clone();
                let armed = enabled.clone();
                let refuse_readback = refusal.clone();
                connections.spawn(async move {
                    if refuse_readback.load(Ordering::SeqCst) && lost.load(Ordering::SeqCst) {
                        return;
                    }
                    let server = UnixStream::connect(socket).await.unwrap();
                    let (mut cr, mut cw) = client.into_split();
                    let (mut sr, mut sw) = server.into_split();
                    let forward = async {
                        let _ = tokio::io::copy(&mut cr, &mut sw).await;
                    };
                    let reply = async {
                        loop {
                            let mut tag = [0];
                            if sr.read_exact(&mut tag).await.is_err() {
                                break;
                            }
                            let mut length = [0; 4];
                            if sr.read_exact(&mut length).await.is_err() {
                                break;
                            }
                            let length_value = u32::from_be_bytes(length) as usize;
                            assert!((4..=16 * 1024 * 1024).contains(&length_value));
                            let mut body = vec![0; length_value - 4];
                            if sr.read_exact(&mut body).await.is_err() {
                                break;
                            }
                            let commit_reply = tag[0] == b'C' && body == b"COMMIT\0";
                            let deferred_fk_failure = drop_commit_failure
                                && tag[0] == b'E'
                                && body.windows(7).any(|field| field == b"C23503\0");
                            if armed.load(Ordering::SeqCst)
                                && (commit_reply || deferred_fk_failure)
                                && !lost.swap(true, Ordering::SeqCst)
                            {
                                break;
                            }
                            if cw.write_all(&tag).await.is_err()
                                || cw.write_all(&length).await.is_err()
                                || cw.write_all(&body).await.is_err()
                            {
                                break;
                            }
                        }
                    };
                    tokio::select! {_=forward=>{},_=reply=>{}}
                });
            }
        });
        Self {
            port,
            lost,
            armed,
            task,
            refuse_readback,
        }
    }
}
