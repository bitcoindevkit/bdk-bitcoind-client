// SPDX-License-Identifier: MIT OR Apache-2.0

//! [`Client`] methods for Bitcoin Core v28.0 and earlier.

use bitcoin::BlockHash;
use corepc_types::{
    bitcoin,
    model::{GetBlockHeaderVerbose, GetBlockVerboseOne},
    v28,
};
use jsonrpc::serde_json::json;

use super::Client;
use crate::{Error, Rpc};

impl Client {
    /// Retrieves the verbose JSON representation of a block header (verbosity 1).
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// The verbose header as a `GetBlockHeaderVerbose` struct.
    pub fn get_block_header_verbose(
        &self,
        block_hash: &BlockHash,
    ) -> Result<GetBlockHeaderVerbose, Error> {
        let header_info: v28::GetBlockHeaderVerbose =
            self.call(Rpc::GetBlockHeader, &[json!(block_hash)])?;
        header_info.into_model().map_err(Error::model)
    }

    /// Retrieves the verbose JSON representation of a block (verbosity 1).
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// The verbose block data as a `GetBlockVerboseOne` struct.
    pub fn get_block_verbose(&self, block_hash: &BlockHash) -> Result<GetBlockVerboseOne, Error> {
        let block_info: v28::GetBlockVerboseOne =
            self.call(Rpc::GetBlock, &[json!(block_hash), json!(1)])?;
        block_info.into_model().map_err(Error::model)
    }
}
