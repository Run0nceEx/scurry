
use std::error::Error;
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

pub trait IntoAddressable<T> {
    fn into_address(self) -> T;
}

impl<T> IntoAddressable<T> for T {
    fn into_address(self) -> T {
        self
    }
}

#[async_trait::async_trait]
/// This can be seen as a constructor/init for `Component` 
/// This is to be used for nonlistening connections
/// and is passed into `ConnectionHandler`
pub trait Connector: Sized {
    /// Constructor
    async fn init_connect<T>(addr: T) -> Result<Self, Box<dyn Error>>
    where T: IntoAddressable<T>;
}


#[async_trait::async_trait]
pub trait Server<Component>: Sized {
    async fn init_bind<A>(addr: A) -> Result<(), Box<dyn Error>> where A: IntoAddressable<A>;
    async fn accept() -> Result<Component, Box<dyn Error>>;
}

#[async_trait::async_trait]
/// Takes whatever Connector<C> constructs
// -- // -- // -- // -- //
pub trait ConnectionHandler<T>: Send + Sync + Sized
{
    /// exec detection
    async fn handle(&self, con: &mut T) -> Result<bool, Box<dyn Error>>;
}



