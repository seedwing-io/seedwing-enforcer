use seedwing_policy_engine::runtime::Response;

/// Rendering a response
pub struct ResponseRenderer<'r, I: IntoIterator<Item = &'r Response>>(pub I);

impl<'r, I> ResponseRenderer<'r, I>
where
    I: IntoIterator<Item = &'r Response>,
{
    pub fn render(self) -> String {
        let mut result = String::new();
        self.render_into(&mut result);
        result
    }

    fn render_into(self, s: &mut String) {
        s.push_str(r#"<ul class="swe-response">"#);
        for r in self.0 {
            s.push_str(&format!(r#"<li class="swe-severity-{}">"#, r.severity));

            s.push_str(&format!(
                r#"
<div class="swe-info">
    <code class="swe-name">{name}</code>
    <span class="swe-severity">({severity})</span>
    <span class="swe-reason">: {reason}</span>
</div>
"#,
                name = r.name,
                severity = r.severity,
                reason = r.reason,
            ));

            ResponseRenderer(&r.rationale).render_into(s);

            s.push_str(r#"</li>"#);
        }
        s.push_str(r#"</ul>"#);
    }
}
