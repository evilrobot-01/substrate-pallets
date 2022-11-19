pub trait QueryDataStorage<Hash, QueryData> {
    fn get_query_data(query_id: Hash) -> QueryData;
    fn store_data(query_data: &QueryData);
}

pub trait RuntimeApi<Hash, Amount> {
    /// Getter function to current oneTime tip by queryId
    fn get_current_tip(query_id: Hash) -> Amount;

    /// Getter function to read current data feeds
    fn get_current_feeds(query_id: Hash) -> Vec<Vec<u8>>;
}
