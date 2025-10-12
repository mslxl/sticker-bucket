use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provider {
    Cpu,
    CoreMl,
    DirectMl,
    Cuda,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ExecutionPolicy {
    #[default]
    Auto,
    Cpu,
    Prefer(Vec<Provider>),
    Require(Provider),
}

pub(crate) fn session_builder(
    policy: &ExecutionPolicy,
) -> Result<ort::session::builder::SessionBuilder> {
    use ort::ep::{CUDA, CoreML, DirectML};

    let providers = match policy {
        ExecutionPolicy::Auto => {
            #[cfg(target_vendor = "apple")]
            let providers = vec![Provider::CoreMl];
            #[cfg(target_os = "windows")]
            let providers = vec![Provider::DirectMl];
            #[cfg(all(not(target_vendor = "apple"), not(target_os = "windows")))]
            let providers = vec![Provider::Cuda];
            providers
        }
        ExecutionPolicy::Cpu => Vec::new(),
        ExecutionPolicy::Prefer(providers) => providers.clone(),
        ExecutionPolicy::Require(provider) => vec![*provider],
    };
    let required = matches!(policy, ExecutionPolicy::Require(_));
    let dispatches = providers
        .into_iter()
        .filter_map(|provider| {
            let dispatch = match provider {
                Provider::Cpu => return None,
                Provider::CoreMl => CoreML::default().build(),
                Provider::DirectMl => DirectML::default().build(),
                Provider::Cuda => CUDA::default().build(),
            };
            Some(if required {
                dispatch.error_on_failure()
            } else {
                dispatch.fail_silently()
            })
        })
        .collect::<Vec<_>>();

    let builder = ort::session::Session::builder()?;
    if dispatches.is_empty() {
        Ok(builder)
    } else {
        builder
            .with_execution_providers(dispatches)
            .map_err(|error| {
                Error::InvalidModelOutput(format!(
                    "failed to configure ONNX execution providers: {error}"
                ))
            })
    }
}
