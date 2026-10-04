use crate::error::CatermError;
use crate::visibility::VisibilityScope;
use async_trait::async_trait;

/// Defines a provider that can supply context for AI operations,
/// honoring visibility scopes.
#[async_trait]
pub trait ContextProvider: Send + Sync {
    /// Name of the provider (e.g., "files", "tasks", "emails")
    fn name(&self) -> &'static str;

    /// Fetches context data, applying the visibility scope rules.
    async fn get_context(&self, scope: &VisibilityScope) -> Result<String, CatermError>;
}

/// Allows registering multiple context providers.
pub type ContextRegistry = Vec<Box<dyn ContextProvider>>;

pub struct ContextManager {
    registry: ContextRegistry,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            registry: Vec::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn ContextProvider>) {
        self.registry.push(provider);
    }

    /// Aggregates context from all registered providers.
    pub async fn aggregate_context(&self, scope: &VisibilityScope) -> String {
        let mut aggregated = String::new();
        for provider in &self.registry {
            if let Ok(ctx) = provider.get_context(scope).await {
                if !ctx.is_empty() {
                    aggregated.push_str(&format!(
                        "\n--- Provider: {} ---\n{}\n",
                        provider.name(),
                        ctx
                    ));
                }
            }
        }
        aggregated
    }
}
