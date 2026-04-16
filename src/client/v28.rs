use bitcoin::BlockHash;
use corepc_types::{
    bitcoin,
    model::{GetBlockHeaderVerbose, GetBlockVerboseOne},
    v28,
};

use jsonrpc::serde_json::json;

use crate::{Client, Error};

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
            self.call("getblockheader", &[json!(block_hash)])?;
        header_info
            .into_model()
            .map_err(Error::GetBlockHeaderVerbose)
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
            self.call("getblock", &[json!(block_hash), json!(1)])?;
        block_info.into_model().map_err(Error::GetBlockVerboseOne)
    }

    /// Retrieves the prune height of the `bitcoind` instance this client is connected to.
    ///
    /// # Returns
    ///
    /// The prune height of the `bitcoind` instance, as an `Option<u32>`.
    pub fn get_prune_height(&self) -> Result<Option<u32>, Error> {
        let res = {
            let res: v28::GetBlockchainInfo = self.call("getblockchaininfo", &[])?;
            res.into_model().map_err(Error::GetBlockchainInfo)?
        };

        if res.pruned {
            Ok(Some(
                res.prune_height.expect("pruned=true implies a pruneheight"),
            ))
        } else {
            Ok(None)
        }
    }
}
