use alloc::string::{String, ToString};

/// Trait that defines the name of an [`Rpc`] method.
pub trait RpcMethod {
    /// Return the RPC method name string.
    fn method(&self) -> String;
}

macro_rules! impl_rpc_methods {
    ( $($name:ident,)+ ) => {
        /// RPCs
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[allow(missing_docs)]
        pub enum Rpc {
            $(
                $name,
            )+
        }

        impl RpcMethod for Rpc {
            fn method(&self) -> String {
                self.to_string()
            }
        }

        impl core::fmt::Display for Rpc {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let s = match self {
                    $(
                        Self::$name => stringify!($name).to_lowercase(),
                    )+
                };

                f.write_str(&s)
            }
        }
    }
}

impl_rpc_methods!(
    GetBlockchainInfo,
    GetBlockCount,
    GetBlockHash,
    // More RPCs ...
);
