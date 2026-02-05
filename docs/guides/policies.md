# AgentKern Policy DSL (Rust Native)

This guide defines the expression syntax for the `agentkern-gate` neuro-symbolic engine.

---

## 🏗️ DSL Grammar

The policy engine uses a high-performance, zero-copy parser implemented in `packages/pillars/gate/src/dsl.rs`.

### Core Syntax
```text
expression   := comparison (('&&' | '||') comparison)*
comparison   := value (OPERATOR value)?
value        := identifier | string | number | boolean
identifier   := 'action' | 'agent_id' | 'context.' KEY
```

### Operators
| Operator | Description | Type Support |
|---|---|---|
| `==` | Equality | String, Number, Bool |
| `!=` | Inequality | String, Number, Bool |
| `>` | Greater Than | Number |
| `<` | Less Than | Number |
| `>=` | Greater/Equal | Number |
| `<=` | Less/Equal | Number |

> [!WARNING]
> Logical operators are parsed strictly: `&&` takes precedence over `||`. Complex nesting (parentheses) is **not supported** in this version for performance reasons.

---

## 📐 Variables & Evaluation Context

The evaluation context (`EvalContext`) exposes three primary signals:

1. **`action`** (String): The action intent (e.g., `transfer_funds`).
2. **`agent_id`** (String): The cryptographic identity of the actor.
3. **`context.*`** (HashMap): Flattened context parameters.

```rust
// Internally maps to:
pub struct EvalContext {
    pub action: String,
    pub agent_id: String,
    pub context: HashMap<String, JsonValue>,
}
```

### Example Usage

```yaml
rules:
  # Check action type
  - id: check-action
    condition: "action == 'transfer_funds'"

  # Check numeric threshold
  - id: check-limit
    condition: "context.amount > 10000"

  # Compound Logic (AND)
  - id: strict-check
    condition: "action == 'delete' && context.resource == 'database'"
    action: deny
```

---

## 🚨 Limitations

1. **No Deep Traversals**: `context.user.address.city` is not supported. Use flattened keys: `context.user_city`.
2. **No Arithmetic**: You cannot do `context.amount * 2`. Calculated values must be passed in the context.
3. **No Regex**: String matching is exact only.

---

## 🛠️ Testing Policies

You can test expressions locally using the `gate-cli`:

```bash
cargo run -p agentkern-gate --bin evaluate -- "action == 'test'"
```
