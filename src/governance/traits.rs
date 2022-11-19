pub trait Oracle<Hash, Timestamp, Account, Amount> {
    /// Removes a value from the oracle.
    fn remove_value(query_id: Hash, timestamp: Timestamp);

    /// Slashes a reporter and transfers their stake amount to the given recipient.
    fn slash_reporter(reporter: &Account, recipient: &Account);

    /// Returns the block number at a given timestamp
    fn get_block_number_by_timestamp(query_id: Hash, timestamp: Timestamp) -> u32;

    /// Returns the address of the reporter who submitted a value for a data ID at a specific time
    fn get_reporter_by_timestamp(query_id: Hash, timestamp: Timestamp) -> Account;

    /// Returns amount required to report oracle values
    fn get_stake_amount() -> Amount;

    /// Allows users to retrieve all information about a staker
    fn get_staker_info(staker: &Account) -> ();

    ///Retrieve value from oracle based on timestamp
    fn retrieve_data(query_id: Hash, timestamp: Timestamp) -> Vec<u8>;
}

pub trait RuntimeApi<Hash, Amount> {
    /// Get the latest dispute fee
    fn get_dispute_fee() -> Amount;

    /// Returns the number of open disputes for a specific query ID
    fn get_open_disputes_on_id(query_id: Hash) -> u8;
}
