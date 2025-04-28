// use tokio::sync::{mpsc, oneshot};


// pub enum WebViewError {

// }

// pub enum WebViewMessage {
//     LoadUrl {
//         url: String,
//         respond_to: oneshot::Sender<Result<(), WebViewError>>
//     },
//     Close {
//         respond_to: oneshot::Sender<Result<(), WebViewError>>
//     },
// }

// struct WebViewActor {
//     receiver: mpsc::Receiver<WebViewMessage>,
//     webview: wry::WebView
// }

// impl WebViewActor {
//     fn new(receiver: mpsc::Receiver<WebViewMessage>, webview: wry::WebView) -> Self {
//         Self {
//             receiver: receiver,
//             webview
//         }
//     }

//     async fn handle_message(&mut self, msg: WebViewMessage) {
//         match msg {
//             WebViewMessage::LoadUrl { url, respond_to} => {
//                 self.webview.load_url(&url);
//                 let _ = respond_to.send(Ok(()));
//             },
//             WebViewMessage::Close { respond_to } => {
//                 let _ = respond_to.send(Ok(()));
//             }
//         }
//     }
// }

// async fn run_webview_actor(mut actor: WebViewActor) {
//     while let Some(msg) = actor.receiver.recv().await {
//         actor.handle_message(msg).await;
//     }
// }

// pub struct WebViewHandle {
//     sender: mpsc::Sender<WebViewMessage>
// }

// impl WebViewHandle {
//     pub fn new(webview: wry::WebView) -> Self {
//         let (sender, receiver) = mpsc::channel(8);
//         let webview_actor = WebViewActor::new(receiver, webview);
//         tokio::spawn(run_webview_actor(webview_actor));

//         Self {
//             sender
//         }
//     }

//     pub async fn load_url(&self, url: String) {
//         let (send, recv) = oneshot::channel();
//         let msg = WebViewMessage::LoadUrl { url: url, respond_to: send };

//         let _ = self.sender.send(msg).await;
//         recv.await.expect("Actor task has been killed")
//     }
// }