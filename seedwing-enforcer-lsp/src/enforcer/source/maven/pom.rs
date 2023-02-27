use url::{ParseError, Url};

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub model_version: String,
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,

    pub dependencies: Dependencies,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependencies {
    #[serde(default)]
    pub dependency: Vec<Dependency>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub group_id: String,
    pub artifact_id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub classifier: Option<String>,
    #[serde(default)]
    pub scope: Scope,
}

impl TryFrom<Dependency> for crate::enforcer::dependency::Dependency {
    type Error = ParseError;

    fn try_from(value: Dependency) -> Result<Self, Self::Error> {
        let mut purl = Url::parse(&format!(
            "pkg:maven/{}/{}@{}",
            value.group_id, value.artifact_id, value.version,
        ))?;

        if let Some(r#type) = &value.r#type {
            purl.query_pairs_mut().append_pair("type", r#type);
        }
        if let Some(classifier) = &value.classifier {
            purl.query_pairs_mut().append_pair("classifier", classifier);
        }

        // FIXME: deal with missing values, like group id or version
        // FIXME: deal with repository URL

        Ok(Self { purl })
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    #[default]
    Compile,
    Provided,
    Runtime,
    Test,
    System,
    Import,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse() {
        let project: Project = quick_xml::de::from_str(
            r#"
<project xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd"
         xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>demo</artifactId>
    <version>1.0-SNAPSHOT</version>

    <dependencies>
        <dependency>
            <groupId>io.quarkus</groupId>
            <artifactId>quarkus-funqy-amazon-lambda</artifactId>
        </dependency>
    </dependencies>

</project>
"#,
        ).unwrap();

        println!("Result: {project:#?}")
    }
}
