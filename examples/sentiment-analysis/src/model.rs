use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thinkwell::{Error, Result};
use vader_sentiment::SentimentIntensityAnalyzer;

/// An exact excerpt from the supplied feedback, chosen by the agent.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextPassage {
    /// Copy a nonempty passage verbatim from the feedback.
    pub text: String,
}

/// VADER's measurements, not probabilities of the author's feelings.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SentimentScores {
    /// Combined sentiment from -1 (most negative) to 1 (most positive).
    pub compound: f64,
    /// Proportion of positive sentiment, from 0 to 1.
    pub positive: f64,
    /// Proportion of negative sentiment, from 0 to 1.
    pub negative: f64,
    /// Proportion of neutral sentiment, from 0 to 1.
    pub neutral: f64,
}

/// The agent's interpretation of the whole document.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Positive,
    Negative,
    Mixed,
    Neutral,
}

/// The agent's short explanation of one passage.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PassageSummary {
    /// A short title for the passage being discussed.
    pub title: String,
    /// Explain its content and tone without claiming sentiment measurements are proof.
    pub summary: String,
}

/// Generated prose is kept separate from the locally recorded measurements.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Analysis {
    /// Interpret the document as a whole; this is not a computed aggregate score.
    pub overall_tone: Tone,
    /// Summaries of the passages considered, with no copied numeric scores.
    pub passages: Vec<PassageSummary>,
    /// A practical recommendation grounded in the supplied feedback.
    pub recommendation: String,
}

pub fn score_passage(
    analyzer: &SentimentIntensityAnalyzer<'_>,
    feedback: &str,
    text: &str,
) -> Result<SentimentScores> {
    if text.trim().is_empty() || !feedback.contains(text) {
        return Err(Error::tool(
            "Choose a nonempty, exact excerpt from the supplied feedback.",
        ));
    }
    scores_from_map(&analyzer.polarity_scores(text))
}

fn scores_from_map(values: &HashMap<&str, f64>) -> Result<SentimentScores> {
    let read = |key, min, max| {
        let value = values
            .get(key)
            .copied()
            .ok_or_else(|| Error::tool("Analyzer omitted a score."))?;
        if !value.is_finite() || !(min..=max).contains(&value) {
            return Err(Error::tool(
                "Analyzer returned a nonfinite or out-of-range score.",
            ));
        }
        Ok(value)
    };
    Ok(SentimentScores {
        compound: read("compound", -1.0, 1.0)?,
        positive: read("pos", 0.0, 1.0)?,
        negative: read("neg", 0.0, 1.0)?,
        neutral: read("neu", 0.0, 1.0)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_scores_distinguish_positive_negative_and_neutral() {
        let analyzer = SentimentIntensityAnalyzer::new();
        for (text, sign) in [
            ("I love this wonderful product!", 1),
            ("This is awful and disappointing.", -1),
            ("The parcel arrived on Tuesday.", 0),
        ] {
            let scores = score_passage(&analyzer, text, text).unwrap();
            match sign {
                1 => assert!(scores.compound > 0.0),
                -1 => assert!(scores.compound < 0.0),
                _ => assert_eq!(scores.compound, 0.0),
            }
        }
    }

    #[test]
    fn only_nonempty_exact_excerpts_are_scored() {
        let analyzer = SentimentIntensityAnalyzer::new();
        let text = "I love this product!";
        assert!(score_passage(&analyzer, text, "love this").is_ok());
        for invalid in ["", " ", "I like this product!", "I LOVE this product!"] {
            assert!(score_passage(&analyzer, text, invalid).is_err());
        }
    }

    #[test]
    fn malformed_analyzer_results_are_errors() {
        let valid = HashMap::from([("compound", 0.5), ("pos", 0.7), ("neg", 0.1), ("neu", 0.2)]);
        assert!(scores_from_map(&valid).is_ok());
        for key in ["compound", "pos", "neg", "neu"] {
            let mut missing = valid.clone();
            missing.remove(key);
            assert!(scores_from_map(&missing).is_err());
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -2.0, 2.0] {
                let mut invalid = valid.clone();
                invalid.insert(key, value);
                assert!(scores_from_map(&invalid).is_err());
            }
        }
    }
}
