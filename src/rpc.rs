// SPDX-License-Identifier: MIT OR Apache-2.0

/// Strongly-typed Bitcoin Core RPC method names.
///
/// Each variant corresponds to a Bitcoin Core JSON-RPC method. The
/// [`Display`](core::fmt::Display) implementation produces the exact lowercase
/// method name string expected by Bitcoin Core.
///
/// These are used internally by [`bitreq::Client`](crate::bitreq::Client) methods
/// and can also be passed directly to [`Client::call`](crate::Client::call)
/// as the `method` argument via [`Rpc::to_string`](std::string::ToString::to_string).
///
/// See <https://bitcoincore.org/en/doc/> for the full RPC reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Rpc {
    /// `getbestblockhash` — returns the hash of the best (tip) block.
    GetBestBlockHash,
    /// `getblock` — returns block data for a given block hash.
    GetBlock,
    /// `getblockcount` — returns the height of the most-work fully-validated chain.
    GetBlockCount,
    /// `getblockfilter` — returns the BIP-158 compact block filter for a block.
    GetBlockFilter,
    /// `getblockhash` — returns the block hash at a given height.
    GetBlockHash,
    /// `getblockheader` — returns block header data for a given block hash.
    GetBlockHeader,
    /// `getrawmempool` — returns all transaction IDs in the memory pool.
    GetRawMempool,
    /// `getrawtransaction` — returns raw transaction data for a given txid.
    GetRawTransaction,
    /// `getblockchaininfo` - returns information about the blockchain state.
    GetBlockchainInfo,
    /// `sendrawtransaction` - send raw transaction to the network.
    SendRawTransaction,
}

impl core::fmt::Display for Rpc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::GetBestBlockHash => "getbestblockhash",
            Self::GetBlock => "getblock",
            Self::GetBlockCount => "getblockcount",
            Self::GetBlockFilter => "getblockfilter",
            Self::GetBlockHash => "getblockhash",
            Self::GetBlockHeader => "getblockheader",
            Self::GetRawMempool => "getrawmempool",
            Self::GetRawTransaction => "getrawtransaction",
            Self::GetBlockchainInfo => "getblockchaininfo",
            Self::SendRawTransaction => "sendrawtransaction",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_display_lowercase() {
        assert_eq!(Rpc::GetBestBlockHash.to_string(), "getbestblockhash");
        assert_eq!(Rpc::GetBlock.to_string(), "getblock");
        assert_eq!(Rpc::GetBlockCount.to_string(), "getblockcount");
        assert_eq!(Rpc::GetBlockFilter.to_string(), "getblockfilter");
        assert_eq!(Rpc::GetBlockHash.to_string(), "getblockhash");
        assert_eq!(Rpc::GetBlockHeader.to_string(), "getblockheader");
        assert_eq!(Rpc::GetRawMempool.to_string(), "getrawmempool");
        assert_eq!(Rpc::GetRawTransaction.to_string(), "getrawtransaction");
        assert_eq!(Rpc::SendRawTransaction.to_string(), "sendrawtransaction");
        assert_eq!(Rpc::GetBlockchainInfo.to_string(), "getblockchaininfo");
    }
}
