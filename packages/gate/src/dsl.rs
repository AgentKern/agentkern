//! VeriMantle-Gate: DSL Expression Parser
//!
//! Simple expression language for policy conditions.
//!
//! # Grammar
//!
//! ```text
//! expression   := comparison (('&&' | '||') comparison)*
//! comparison   := value (('==' | '!=' | '>' | '<' | '>=' | '<=') value)?
//! value        := identifier | string | number | boolean
//! identifier   := ('action' | 'agent_id' | 'context.' path)
//! ```
//!
//! # Examples
//!
//! - `action == 'transfer_funds'`
//! - `context.amount > 10000`
//! - `action == 'delete' && context.resource == 'database'`

use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Context for evaluating expressions.
#[derive(Debug, Clone)]
pub struct EvalContext {
    pub action: String,
    pub agent_id: String,
    pub context: HashMap<String, JsonValue>,
}

/// Evaluate a condition expression against the given context.
/// Returns true if the condition matches.
pub fn evaluate(condition: &str, ctx: &EvalContext) -> bool {
    // Split by logical operators (&&, ||)
    let parts: Vec<&str> = condition.split("&&").collect();
    
    if parts.len() > 1 {
        // AND logic - all parts must be true
        return parts.iter().all(|part| evaluate_single(part.trim(), ctx));
    }
    
    let parts: Vec<&str> = condition.split("||").collect();
    if parts.len() > 1 {
        // OR logic - any part can be true
        return parts.iter().any(|part| evaluate_single(part.trim(), ctx));
    }
    
    evaluate_single(condition, ctx)
}

/// Evaluate a single comparison expression.
fn evaluate_single(expr: &str, ctx: &EvalContext) -> bool {
    // Parse comparison operators
    let operators = ["==", "!=", ">=", "<=", ">", "<"];
    
    for op in operators {
        if let Some(idx) = expr.find(op) {
            let left = expr[..idx].trim();
            let right = expr[idx + op.len()..].trim();
            
            let left_val = resolve_value(left, ctx);
            let right_val = resolve_value(right, ctx);
            
            return match op {
                "==" => values_equal(&left_val, &right_val),
                "!=" => !values_equal(&left_val, &right_val),
                ">" => compare_values(&left_val, &right_val) == std::cmp::Ordering::Greater,
                "<" => compare_values(&left_val, &right_val) == std::cmp::Ordering::Less,
                ">=" => {
                    let cmp = compare_values(&left_val, &right_val);
                    cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal
                }
                "<=" => {
                    let cmp = compare_values(&left_val, &right_val);
                    cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal
                }
                _ => false,
            };
        }
    }
    
    // No operator found - check if it's a truthy value
    let val = resolve_value(expr, ctx);
    is_truthy(&val)
}

/// Resolve a value from the context or parse as literal.
fn resolve_value(token: &str, ctx: &EvalContext) -> JsonValue {
    let token = token.trim();
    
    // Check for built-in identifiers
    match token {
        "action" => return JsonValue::String(ctx.action.clone()),
        "agent_id" => return JsonValue::String(ctx.agent_id.clone()),
        _ => {}
    }
    
    // Check for context.* path
    if let Some(path) = token.strip_prefix("context.") {
        return ctx.context.get(path).cloned().unwrap_or(JsonValue::Null);
    }
    
    // Parse as literal
    // String literal (with quotes)
    if (token.starts_with('\'') && token.ends_with('\''))
        || (token.starts_with('"') && token.ends_with('"'))
    {
        return JsonValue::String(token[1..token.len() - 1].to_string());
    }
    
    // Boolean literal
    match token.to_lowercase().as_str() {
        "true" => return JsonValue::Bool(true),
        "false" => return JsonValue::Bool(false),
        "null" => return JsonValue::Null,
        _ => {}
    }
    
    // Number literal
    if let Ok(n) = token.parse::<i64>() {
        return JsonValue::Number(n.into());
    }
    if let Ok(n) = token.parse::<f64>() {
        return serde_json::Number::from_f64(n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null);
    }
    
    JsonValue::Null
}

fn values_equal(a: &JsonValue, b: &JsonValue) -> bool {
    a == b
}

fn compare_values(a: &JsonValue, b: &JsonValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    
    match (a, b) {
        (JsonValue::Number(a), JsonValue::Number(b)) => {
            let a_f = a.as_f64().unwrap_or(0.0);
            let b_f = b.as_f64().unwrap_or(0.0);
            a_f.partial_cmp(&b_f).unwrap_or(Ordering::Equal)
        }
        (JsonValue::String(a), JsonValue::String(b)) => a.cmp(b),
        _ => Ordering::Equal,
    }
}

fn is_truthy(val: &JsonValue) -> bool {
    match val {
        JsonValue::Null => false,
        JsonValue::Bool(b) => *b,
        JsonValue::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        JsonValue::String(s) => !s.is_empty(),
        JsonValue::Array(a) => !a.is_empty(),
        JsonValue::Object(o) => !o.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(action: &str, amount: i64) -> EvalContext {
        let mut context = HashMap::new();
        context.insert("amount".to_string(), JsonValue::Number(amount.into()));
        
        EvalContext {
            action: action.to_string(),
            agent_id: "test-agent".to_string(),
            context,
        }
    }

    #[test]
    fn test_action_equals() {
        let ctx = make_ctx("transfer_funds", 5000);
        assert!(evaluate("action == 'transfer_funds'", &ctx));
        assert!(!evaluate("action == 'send_email'", &ctx));
    }

    #[test]
    fn test_numeric_comparison() {
        let ctx = make_ctx("transfer_funds", 15000);
        assert!(evaluate("context.amount > 10000", &ctx));
        assert!(!evaluate("context.amount < 10000", &ctx));
        assert!(evaluate("context.amount >= 15000", &ctx));
        assert!(evaluate("context.amount <= 15000", &ctx));
    }

    #[test]
    fn test_and_logic() {
        let ctx = make_ctx("transfer_funds", 15000);
        assert!(evaluate("action == 'transfer_funds' && context.amount > 10000", &ctx));
        assert!(!evaluate("action == 'send_email' && context.amount > 10000", &ctx));
    }

    #[test]
    fn test_or_logic() {
        let ctx = make_ctx("send_email", 100);
        assert!(evaluate("action == 'send_email' || action == 'transfer_funds'", &ctx));
        assert!(!evaluate("action == 'delete' || action == 'drop'", &ctx));
    }
}
