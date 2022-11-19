pub trait Governance<Account> {
    /// Returns the total number of votes
    fn get_vote_count() -> u32;

    /// Returns the total number of votes cast by an address
    fn get_vote_tally_by_address(address: &Account) -> u32;
}

pub trait RuntimeApi<Hash, Block, Reporter> {
    /// Retrieves the next value for the queryId after the specified timestamp
    fn get_data_after(query_id: Hash, timestamp: Block) -> (Vec<u8>, Block);

    /// Retrieves the latest value for the queryId before the specified timestamp
    fn get_data_before(query_id: Hash, timestamp: Block) -> (Vec<u8>, Block);

    /// Returns the multiple most recent values for a given queryId before a given timestamp.
    fn get_multiple_values_before(
        query_id: Hash,
        timestamp: Block,
        max_age: u8,
        max_count: u8,
    ) -> (Vec<Vec<u8>>, Vec<Block>);

    /// Retrieves the address of the reporter for a given queryId and timestamp.
    fn get_reporter_by_timestamp(query_id: Hash, timestamp: Block) -> Reporter;

    /// Determines whether a specific value with a given queryId and timestamp has been disputed
    fn is_in_dispute(query_id: Hash, timestamp: Block) -> bool;

    /// Retrieves a specific value by queryId and timestamp
    fn retrieve_data(query_id: Hash, timestamp: Block) -> Vec<u8>;
}
