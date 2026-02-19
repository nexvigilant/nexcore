// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! MCP tool generation for prima-academy.
//!
//! ## Tier: T2-C (μ + → + π)
//!
//! Exposes academic course classification as MCP tools.

use prima_academy::{Course, CourseLevel, PrimaTier, Subject};
use serde_json::{Value as JsonValue, json};

/// Generate MCP tool definitions for prima-academy.
#[must_use]
pub fn academy_tools(prefix: &str) -> Vec<JsonValue> {
    vec![
        json!({
            "name": format!("{}_course_parse", prefix),
            "description": "Parse a course code (e.g., 'MTH 101') and return its classification",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Course code like 'MTH 101', 'PHY201', 'CSC 350'"
                    }
                },
                "required": ["code"]
            }
        }),
        json!({
            "name": format!("{}_course_tier", prefix),
            "description": "Get the Prima tier for a course level (Knowledge Funnel Inversion)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "level": {
                        "type": "integer",
                        "description": "Course level 1-7 (1=Freshman, 5=Graduate, 6=Doctoral)",
                        "minimum": 1,
                        "maximum": 7
                    }
                },
                "required": ["level"]
            }
        }),
        json!({
            "name": format!("{}_transfer_confidence", prefix),
            "description": "Get transfer confidence for a Prima tier (T1=1.0, T2-P=0.9, T2-C=0.7, T3=0.4)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tier": {
                        "type": "string",
                        "description": "Prima tier: T1, T2-P, T2-C, or T3",
                        "enum": ["T1", "T2-P", "T2-C", "T3"]
                    }
                },
                "required": ["tier"]
            }
        }),
        json!({
            "name": format!("{}_subject_classify", prefix),
            "description": "Classify a subject code and check if STEM/Healthcare",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Subject code like 'MTH', 'PHY', 'CSC', 'PHR'"
                    }
                },
                "required": ["code"]
            }
        }),
        json!({
            "name": format!("{}_knowledge_funnel", prefix),
            "description": "Map full course progression to Prima tiers (100→T3, 200→T3, 300→T2-C, 400→T2-C, 500→T2-P, 600→T1)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "subject": {
                        "type": "string",
                        "description": "Subject code"
                    }
                },
                "required": ["subject"]
            }
        }),
    ]
}

/// Execute an academy tool call.
pub fn execute_academy_tool(name: &str, args: &JsonValue) -> Option<JsonValue> {
    // Match on tool name suffix
    if name.ends_with("_course_parse") {
        let code = args.get("code")?.as_str()?;
        let course = Course::parse(code).ok()?;
        let level = course.level();
        let tier = course.prima_tier();

        Some(json!({
            "code": course.code(),
            "subject": format!("{:?}", course.subject),
            "number": course.number,
            "level": {
                "name": level.year_name(),
                "number": level.as_number()
            },
            "prima_tier": {
                "symbol": tier.symbol(),
                "transfer_confidence": tier.transfer_confidence()
            },
            "is_graduate": course.is_graduate()
        }))
    } else if name.ends_with("_course_tier") {
        let level_num = args.get("level")?.as_u64()? as u8;
        let level = CourseLevel::from_digit(level_num).ok()?;
        let tier = level.to_prima_tier();

        Some(json!({
            "level": {
                "number": level.as_number(),
                "name": level.year_name()
            },
            "prima_tier": {
                "symbol": tier.symbol(),
                "transfer_confidence": tier.transfer_confidence()
            },
            "inversion_note": "Higher academic level → Lower Prima tier → Higher transfer confidence"
        }))
    } else if name.ends_with("_transfer_confidence") {
        let tier_str = args.get("tier")?.as_str()?;
        let tier = match tier_str {
            "T1" => PrimaTier::T1,
            "T2-P" => PrimaTier::T2P,
            "T2-C" => PrimaTier::T2C,
            "T3" => PrimaTier::T3,
            _ => return None,
        };

        Some(json!({
            "tier": tier.symbol(),
            "transfer_confidence": tier.transfer_confidence(),
            "meaning": match tier {
                PrimaTier::T1 => "Universal primitive - transfers perfectly across all domains",
                PrimaTier::T2P => "Cross-domain primitive - high transfer with structural mapping",
                PrimaTier::T2C => "Cross-domain composite - moderate transfer with adaptation",
                PrimaTier::T3 => "Domain-specific - limited transfer, requires translation",
            }
        }))
    } else if name.ends_with("_subject_classify") {
        let code = args.get("code")?.as_str()?;
        let subject = Subject::from_code(code);

        Some(json!({
            "code": subject.code(),
            "subject": format!("{:?}", subject),
            "is_stem": subject.is_stem(),
            "is_healthcare": subject.is_healthcare()
        }))
    } else if name.ends_with("_knowledge_funnel") {
        let subject_code = args.get("subject")?.as_str()?;
        let subject = Subject::from_code(subject_code);

        let levels = [
            (100, CourseLevel::Introductory),
            (200, CourseLevel::Intermediate),
            (300, CourseLevel::Advanced),
            (400, CourseLevel::Capstone),
            (500, CourseLevel::Graduate),
            (600, CourseLevel::Doctoral),
        ];

        let progression: Vec<JsonValue> = levels
            .iter()
            .map(|(num, level)| {
                let tier = level.to_prima_tier();
                json!({
                    "course": format!("{} {}", subject.code(), num),
                    "level": level.year_name(),
                    "tier": tier.symbol(),
                    "transfer": tier.transfer_confidence()
                })
            })
            .collect();

        Some(json!({
            "subject": subject.code(),
            "funnel": progression,
            "principle": "Knowledge Funnel Inversion: Higher academic level → Lower Prima tier → Higher transfer confidence"
        }))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_academy_tools_count() {
        let tools = academy_tools("prima");
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_execute_course_parse() {
        let result = execute_academy_tool("prima_course_parse", &json!({"code": "MTH 101"}));
        assert!(result.is_some());
        let r = result.unwrap_or(json!({}));
        assert_eq!(r["code"], "MTH 101");
        assert_eq!(r["prima_tier"]["symbol"], "T3");
    }

    #[test]
    fn test_execute_course_tier() {
        let result = execute_academy_tool("prima_course_tier", &json!({"level": 6}));
        assert!(result.is_some());
        let r = result.unwrap_or(json!({}));
        assert_eq!(r["prima_tier"]["symbol"], "T1");
    }

    #[test]
    fn test_execute_transfer_confidence() {
        let result = execute_academy_tool("prima_transfer_confidence", &json!({"tier": "T1"}));
        assert!(result.is_some());
        let r = result.unwrap_or(json!({}));
        assert_eq!(r["transfer_confidence"], 1.0);
    }

    #[test]
    fn test_execute_subject_classify() {
        let result = execute_academy_tool("prima_subject_classify", &json!({"code": "PHR"}));
        assert!(result.is_some());
        let r = result.unwrap_or(json!({}));
        assert!(r["is_healthcare"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_execute_knowledge_funnel() {
        let result = execute_academy_tool("prima_knowledge_funnel", &json!({"subject": "MTH"}));
        assert!(result.is_some());
        let r = result.unwrap_or(json!({}));
        let funnel = r["funnel"].as_array();
        assert!(funnel.is_some());
        assert_eq!(funnel.map(|f| f.len()).unwrap_or(0), 6);
    }
}
