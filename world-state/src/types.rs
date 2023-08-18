use rlp::{Rlp, Decodable, DecoderError};
use serde::{Serialize, Deserialize};

use crate::{transaction::Transaction, cron::Cron, metadata::Metadata, metacontract::MetaContract, transaction_receipt::TransactionReceipt};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrieResult {
  pub success: bool,
  pub result: Option<String>,
}

pub enum DecodableEnum {
  Transaction(Result<Transaction, DecoderError>),
  Cron(Result<Cron, DecoderError>),
  Metadata(Result<Metadata, DecoderError>),
  MetaContract(Result<MetaContract, DecoderError>),
  Receipt(Result<TransactionReceipt, DecoderError>),
}

impl DecodableEnum {
  pub fn decode(trie_key: &str, rlp: &Rlp) -> Option<Self> {
    match trie_key {
      "tx" => Some(DecodableEnum::Transaction(Transaction::decode(rlp))),
      "cron" => Some(DecodableEnum::Cron(Cron::decode(rlp))),
      "metadata" => Some(DecodableEnum::Metadata(Metadata::decode(rlp))),
      "metacontract" => Some(DecodableEnum::MetaContract(MetaContract::decode(rlp))),
      "receipt" => Some(DecodableEnum::Receipt(TransactionReceipt::decode(rlp))),
      _ => None,
    }
  }
}