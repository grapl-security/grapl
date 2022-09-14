mod plugin_work_processor;
pub use plugin_work_processor::{
    PluginWorkProcessor,
    Workload,
};

mod analyzer_work_processor;
pub use analyzer_work_processor::AnalyzerWorkProcessor;

mod generator_work_processor;
pub use generator_work_processor::GeneratorWorkProcessor;
