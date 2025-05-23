use std::collections::HashMap;

pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&mut self, context: &mut PluginContext);
}

pub struct PluginContext {
    pub selected_text: Option<String>,
    pub current_file: Option<String>,
    pub cursor_position: (usize, usize),
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    pub fn execute_plugin(&mut self, name: &str, context: &mut PluginContext) {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.execute(context);
        }
    }

    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// Example plugin implementation
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example"
    }

    fn execute(&mut self, context: &mut PluginContext) {
        if let Some(text) = &context.selected_text {
            println!("Selected text: {}", text);
        }
    }
}
