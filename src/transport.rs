
// /// Implementation of a stdio transport for MCP.
// pub struct StdioTransport {
//     sender: tokio::sync::mpsc::Sender<JsonRpcMessage>,
//     on_message: tokio::sync::Mutex<Option<Box<dyn Fn(JsonRpcMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>>>,
//     on_error: tokio::sync::Mutex<Option<Box<dyn Fn(ProtocolError) + Send + Sync>>>,
//     on_close: tokio::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
// }

// impl StdioTransport {
//     /// Creates a new stdio transport.
//     pub fn new() -> Self {
//         let (tx, _) = tokio::sync::mpsc::channel(100);
//         StdioTransport {
//             sender: tx,
//             on_message: tokio::sync::Mutex::new(None),
//             on_error: tokio::sync::Mutex::new(None),
//             on_close: tokio::sync::Mutex::new(None),
//         }
//     }
// }

// impl Transport for StdioTransport {
//     fn start(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProtocolError>> + Send>> {
//         Box::pin(async {
//             // Implementation for reading from stdin and writing to stdout
//             Ok(())
//         })
//     }

//     fn send(&self, message: JsonRpcMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProtocolError>> + Send>> {
//         let sender = self.sender.clone();
//         Box::pin(async move {
//             sender
//                 .send(message)
//                 .await
//                 .map_err(|e| ProtocolError::TransportError(format!("Failed to send message: {}", e)))
//         })
//     }

//     fn close(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProtocolError>> + Send>> {
//         Box::pin(async { Ok(()) })
//     }

//     fn on_message(&self, handler: Box<dyn Fn(JsonRpcMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>) {
//         let on_message = self.on_message.clone();
//         tokio::spawn(async move {
//             let mut guard = on_message.lock().await;
//             *guard = Some(handler);
//         });
//     }

//     fn on_error(&self, handler: Box<dyn Fn(ProtocolError) + Send + Sync>) {
//         let on_error = self.on_error.clone();
//         tokio::spawn(async move {
//             let mut guard = on_error.lock().await;
//             *guard = Some(handler);
//         });
//     }

//     fn on_close(&self, handler: Box<dyn Fn() + Send + Sync>) {
//         let on_close = self.on_close.clone();
//         tokio::spawn(async move {
//             let mut guard = on_close.lock().await;
//             *guard = Some(handler);
//         });
//     }
// }

// /// Trait for MCP transport implementations.
// #[async_trait]
// pub trait Transport: Send + Sync {
//     /// Starts the transport and begins processing messages.
//     async fn start(&self) -> Result<(), ProtocolError>;

//     /// Sends a JSON-RPC message through the transport.
//     async fn send(&self, message: JsonRpcMessage) -> Result<(), ProtocolError>;

//     /// Closes the transport connection.
//     async fn close(&self) -> Result<(), ProtocolError>;

//     /// Sets a handler for incoming messages.
//     fn on_message(&self, handler: Box<dyn Fn(JsonRpcMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>);

//     /// Sets a handler for transport errors.
//     fn on_error(&self, handler: Box<dyn Fn(ProtocolError) + Send + Sync>);

//     /// Sets a handler for transport close events.
//     fn on_close(&self, handler: Box<dyn Fn() + Send + Sync>);
// }
