use filters::FilterSettings;

pub mod filters;

#[derive(Debug)]
pub enum MessageType {
    FilterSettings(FilterSettings),
}
