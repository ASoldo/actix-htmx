use futures::lock::Mutex;
pub struct MySanityConfig {
    pub sanity_config: Mutex<sanity::SanityConfig>,
}
