use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DiscoverError {
    #[error("Invalid argument: `{0}`. Error message: `{1}`")]
    MalformedArgument(String, String),
    #[error("Duplicate argument with key: `{0}`")]
    DuplicateArgument(String),
    #[error("Argument with key: `{0}` was not expected")]
    UnexpectedArgument(String),
    #[error("Argument with key: `{0}` is required")]
    MissingArgument(String),
    #[error("Unbale to retrieve data from provider. Error message: `{0}`")]
    ProviderRequestFailed(String),
    #[error(
        "Unsupported provider `{0}`. Either the provider is not supported or it is not enabled."
    )]
    UnsupportedProvider(String),
}
