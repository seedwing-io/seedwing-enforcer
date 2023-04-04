use crate::command::once::{AggregatedResult, NamesAreHard};
use seedwing_enforcer_common::enforcer::Outcome;
use seedwing_enforcer_common::enforcer::Severity;

pub fn result_to_markdown(data: &NamesAreHard) -> String {
    let mut markdown = String::new();

    // Overall status and title
    markdown.push_str("# Seedwing Enforcer Dependency analysis\n\n");

    match &data.status {
        AggregatedResult::Accepted => markdown.push_str("# ✔ Accepted ✔️\n"),
        AggregatedResult::Rejected => markdown.push_str("# ❌ Rejected ❌\n"),
        AggregatedResult::ConfigError(_) => {
            markdown.push_str("# Configuration Error ❌\n");
            return markdown;
        }
    }

    markdown.push_str("\n\n");

    // Define table header
    markdown.push_str("| Satisfied | Package URL | Reason |\n");
    markdown.push_str("| --------- | ----------- | ------ |\n");

    // Populate the table with dependencies
    for result in &data.details {
        match &result.outcome {
            Outcome::Ok => markdown.push_str(&format!("| ✔ | {} | | \n", &result.dependency.purl)),
            Outcome::RejectedHtml(_) => unreachable!(),
            Outcome::RejectedRaw(resp) => {
                let resp = resp.clone().collapse(Severity::Error);
                let reason = resp.reason.clone();
                let pattern = resp.name.clone();
                println!("{}", serde_yaml::to_string(&resp).unwrap());
                markdown.push_str(&format!(
                    "| ❌ | {} |{pattern} : {reason} | \n",
                    &result.dependency.purl
                ))
            }
        }
    }

    markdown
}
