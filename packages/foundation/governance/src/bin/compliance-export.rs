use agentkern_governance::ai::eu_ai_act::{EuAiActExporter, TechnicalDocumentation};
use std::fs;

fn main() {
    let exporter = EuAiActExporter::new();
    let sample_doc = TechnicalDocumentation::sample();
    let report = exporter.export_text(&sample_doc);

    fs::write("EU_AI_ACT_REPORT.txt", report).expect("Unable to write report");
    println!("✅ EU AI Act Compliance Report generated: EU_AI_ACT_REPORT.txt");
}
