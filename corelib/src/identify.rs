
use std::error::Error;
// CPE                          (Common platform enumeration)
// nmap-service-probes          (Probe message)
// nmap-protocols               (Official ICANNA port numbers)
// nmap-payloads                (Host discovery)
// nmap-mac-prefixes            
// nmap-host-to-db              (Host discovery to fingerprint)
// nmap-rpc                     (Remote proceedure calls)



// #[async_trait::async_trait]
// pub trait Negotiate: Sized + 'static {

//     async fn negotiate<T>(&self, addr: SocketAddr, timeout: Duration, protocol: T) -> Result<bool, Box<dyn Error>>
//     where 
//         T: ProtocolIdentifier<Self> + Send + Sync,
//         Self: Connector + Send
//     {   
//         let mut stream = Self::init_connect(addr).await?;
//         Ok(timeout_future(timeout, protocol.detect(&mut stream)).await??)
//     }

//     async fn noop(&self, addr: SocketAddr) -> Result<(), Box<dyn Error>>
//     where
//         Self: Connector + Send
//     {
//         Self::init_connect(addr).await?;
//         Ok(())
//     }
// }
// Blanket expression for any type that has Connector<T> + Identifier<T>
// impl<T> Negotiate for T 
// where 
//     T: Connector + ProtocolIdentifier<T> + Send + Sync + 'static {}
// pub use tokio::net::TcpStream;
// impl Negotiate for TcpStream {}


// Handlers can do a variety of tasks
//
// Connector<Layer/Component>             - Understands how to connect with T (Layer)
// IndentifyProtocol<T: Connector<T>>     - A hand shake handler for managing how connections work

#[async_trait::async_trait]
/// This can be seen as a constructor/init for `Component` 
/// This is to be used for nonlistening connections
/// and is passed into `ConnectionHandler`
pub trait Connector<Component> {
    /// Constructor
    async fn init_connect<T>(addr: T) -> Result<Component, Box<dyn Error>>;
}


#[async_trait::async_trait]
pub trait ServerConnector<Component> {
    async fn init_bind<A>(addr: A) -> Result<Component, Box<dyn Error>>;
}

#[async_trait::async_trait]
/// This can be seen as a constructor/init for `T`
/// and is passed into `ConnectionHandler`
pub trait Server<T: ServerConnector<T>> {
    async fn accept() -> Result<T, Box<dyn Error>>;
}


#[async_trait::async_trait]
/// Probes service (Single request) with data that shouldn't cause change of state or alert an IDS
/// Attemping to Identify protocols

/// Takes whatever Connector<C> constructs
// -- // -- // -- // -- //
//
pub trait ConnectionHandler<T>
{
    /// exec detection
    async fn handle(&self, con: &mut T) -> Result<bool, Box<dyn Error>>;
}



