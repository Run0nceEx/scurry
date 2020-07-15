use crate::error::Error;
// CPE                          (Common platform enumeration)
// nmap-service-probes          (Probe message)
// nmap-protocols               (Official ICANNA port numbers)
// nmap-payloads                (Host discovery)
// nmap-mac-prefixes            
// nmap-host-to-db              (Host discovery to fingerprint)
// nmap-rpc                     (Remote proceedure calls)
///////////////////
// Handlers can do a variety of tasks
//
// Connector<Layer/Component>             - Understands how to connect with T (Layer)
// IndentifyProtocol<T: Connector<T>>     - A hand shake handler for managing how connections work


#[async_trait::async_trait]
/// This can be seen as a constructor/init for `Component` 
/// This is to be used for nonlistening connections
/// and is passed into `ConnectionHandler`
pub trait Connector<T, A>: Sized {
    /// Constructor
    async fn init_connect(addr: A) -> Result<T, Error>;

    async fn close_connection(con: T) -> Result<(), Error>;
}


#[async_trait::async_trait]
pub trait Server<T, A>: Sized {
    async fn init_bind(addr: A) -> Result<(), Error>;

    async fn accept() -> Result<T, Error>;
}


#[async_trait::async_trait]
/// Takes whatever Connector<C> constructs
// -- // -- // -- // -- //
pub trait ConnectionHandler<T>: Send + Sync + Sized
{
    /// exec detection
    async fn handle(&self, con: &mut T) -> Result<bool, Error>;
}



