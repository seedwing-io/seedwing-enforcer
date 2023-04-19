use crate::command::once::{AggregatedResult, NamesAreHard};
use seedwing_policy_engine::lang::Severity;
use seedwing_policy_engine::runtime::response::Collector;

pub fn result_to_markdown(data: &NamesAreHard) -> String {
    let mut markdown = String::new();

    // Overall status and title
    markdown.push_str("# Seedwing Enforcer Dependency analysis\n\n");

    match &data.status {
        AggregatedResult::Accepted => markdown.push_str("✔ Accepted ✔️\n"),
        AggregatedResult::Rejected => markdown.push_str("❌ Rejected ❌\n"),
        AggregatedResult::ConfigError(_) => {
            markdown.push_str("Configuration Error ❌\n");
            return markdown;
        }
    }

    markdown.push_str("\n\n");

    // Define table header
    markdown.push_str("| Satisfied | Package URL | Reason |\n");
    markdown.push_str("| --------- | ----------- | ------ |\n");

    // Populate the table with dependencies
    for result in &data.details {
        match result.response.severity {
            Severity::None => markdown.push_str(&format!(
                "| {} | {} | | \n",
                severity_as_emoji(Severity::None),
                &result.dependency.purl
            )),
            severity => {
                let resp = Collector::new(&result.response)
                    .highest_severity()
                    .collect();

                let reasons = resp
                    .into_iter()
                    .map(|resp| {
                        format!(
                            "`{name}` : {reason}",
                            name = resp.name,
                            reason = resp.reason
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("<br>");

                markdown.push_str(&format!(
                    "| {} | {} | {reasons} | \n",
                    severity_as_emoji(severity),
                    &result.dependency.purl
                ))
            }
        }
    }

    markdown
}

fn severity_as_emoji(severity: Severity) -> &'static str {
    match severity {
        Severity::None => "✔",
        Severity::Advice => "💡",
        Severity::Warning => "⚠️",
        Severity::Error => "❌",
    }
}
