//! The webview's Content-Security-Policy, read from `tauri.conf.json`.
//!
//! Requirement carried here: REQ-cj5jmx (implements RC-mq2365, which mitigates
//! HAZ-e2zupz). The policy is configuration, not code, so this file holds the
//! checker as well as the tests: the abnormal cases prove the checker rejects
//! each weakening the requirement forbids, and the normal cases prove the
//! shipped configuration passes it.
//!
//! What this file cannot show: that the bundled app still loads under the
//! policy. Tauri appends hashes of the bundle's inline scripts to `script-src`
//! at build time, and nothing in the gate builds the bundle. That is the
//! owner's smoke test (`npm --prefix app run tauri build`), recorded as a
//! verification limitation in the risk register.

use std::collections::BTreeMap;

/// The only network origins the shipped policy may name: Tauri's IPC
/// endpoints (`ipc:` on macOS and Linux, `http://ipc.localhost` on Windows).
const IPC_SOURCES: [&str; 2] = ["ipc:", "http://ipc.localhost"];

/// The development server's origins, which only `devCsp` may add.
const DEV_SERVER_SOURCES: [&str; 2] = ["http://localhost:5173", "ws://localhost:5173"];

/// Directives that must be present with exactly `'none'`.
const NONE_DIRECTIVES: [&str; 4] = ["object-src", "base-uri", "frame-ancestors", "form-action"];

/// The only directive names the policy may use. Any other is refused, so a
/// directive that overrides `script-src` for some scripts (`script-src-elem`,
/// `script-src-attr`, `worker-src`) cannot reopen what `script-src` closes.
const PERMITTED_DIRECTIVES: [&str; 9] = [
    "default-src",
    "script-src",
    "style-src",
    "img-src",
    "connect-src",
    "object-src",
    "base-uri",
    "frame-ancestors",
    "form-action",
];

type Directives = BTreeMap<String, Vec<String>>;

/// Parse a policy string into directive -> sources. Names and sources are
/// lower-cased (both are case-insensitive in CSP). A repeated directive is an
/// error: browsers honour the first and ignore the rest, so a checker that read
/// the second would pass a policy the webview never applies.
fn parse(policy: &str) -> Result<Directives, String> {
    let mut out = Directives::new();
    for part in policy.split(';') {
        let mut tokens = part.split_ascii_whitespace().map(str::to_ascii_lowercase);
        let Some(name) = tokens.next() else { continue };
        if out.contains_key(&name) {
            return Err(format!("directive `{name}` is repeated"));
        }
        out.insert(name, tokens.collect());
    }
    Ok(out)
}

/// A hash or nonce source: names one script exactly, so it opens nothing.
fn is_hash_or_nonce(src: &str) -> bool {
    ["'sha256-", "'sha384-", "'sha512-", "'nonce-"]
        .iter()
        .any(|p| src.starts_with(p))
}

/// A source that names a network origin: a host, a wildcard, or a network
/// scheme. Keywords (`'self'`, `'none'`, ...), hashes, nonces and the
/// non-network schemes `data:` and `blob:` are not.
fn is_network_source(src: &str) -> bool {
    !(src.starts_with('\'') || src == "data:" || src == "blob:")
}

/// Check a `tauri.conf.json` CSP value against every clause of REQ-cj5jmx.
/// `extra_origins` are network origins tolerated beyond Tauri IPC — empty for
/// the shipped policy, the dev server's for `devCsp`.
fn check(value: &serde_json::Value, extra_origins: &[&str]) -> Result<(), String> {
    let policy = value
        .as_str()
        .ok_or_else(|| format!("policy must be a string, found {value}"))?;
    let d = parse(policy)?;
    let sources = |name: &str| {
        d.get(name)
            .ok_or_else(|| format!("directive `{name}` is missing"))
    };
    let tolerated_origin =
        |src: &String| IPC_SOURCES.contains(&src.as_str()) || extra_origins.contains(&src.as_str());

    if let Some(name) = d.keys().find(|n| !PERMITTED_DIRECTIVES.contains(&n.as_str())) {
        return Err(format!("directive `{name}` is not permitted"));
    }

    if sources("default-src")? != &["'self'"] {
        return Err("default-src must be exactly 'self'".into());
    }

    for src in sources("script-src")? {
        let allowed = src == "'self'"
            || is_hash_or_nonce(src)
            || extra_origins.contains(&src.as_str());
        if !allowed {
            return Err(format!("script-src allows `{src}`"));
        }
    }

    for name in NONE_DIRECTIVES {
        if sources(name)? != &["'none'"] {
            return Err(format!("{name} must be exactly 'none'"));
        }
    }

    for src in sources("connect-src")? {
        if !tolerated_origin(src) {
            return Err(format!("connect-src allows `{src}`"));
        }
    }

    for (name, srcs) in &d {
        for src in srcs {
            if is_network_source(src) && !tolerated_origin(src) {
                return Err(format!("{name} names the network origin `{src}`"));
            }
        }
    }
    Ok(())
}

fn security() -> serde_json::Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");
    let text = std::fs::read_to_string(path).expect("read tauri.conf.json");
    let conf: serde_json::Value = serde_json::from_str(&text).expect("parse tauri.conf.json");
    conf["app"]["security"].clone()
}

/// The policy the plan prescribes, as a fixture the abnormal cases weaken one
/// clause at a time. It is not read from the configuration, so each abnormal
/// case fails for its own mutation alone.
const FIXTURE: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
    img-src 'self' data:; connect-src ipc: http://ipc.localhost; object-src 'none'; \
    base-uri 'none'; frame-ancestors 'none'; form-action 'none'";

/// Check `policy` as the shipped `csp`: no origin beyond Tauri IPC.
fn check_as_shipped(policy: &str) -> Result<(), String> {
    check(&serde_json::Value::String(policy.into()), &[])
}

/// Check `policy` as the `devCsp`: the dev server's origins are tolerated too.
fn check_as_dev(policy: &str) -> Result<(), String> {
    check(&serde_json::Value::String(policy.into()), &DEV_SERVER_SOURCES)
}

/// Check that `dev` differs from `shipped` only by adding the dev server's
/// origins: the same directives, each with the same sources, except that
/// `script-src` and `connect-src` each add exactly `DEV_SERVER_SOURCES`.
fn check_dev_difference(shipped: &str, dev: &str) -> Result<(), String> {
    let mut expected = parse(shipped)?;
    for name in ["script-src", "connect-src"] {
        expected
            .get_mut(name)
            .ok_or_else(|| format!("shipped policy lacks `{name}`"))?
            .extend(DEV_SERVER_SOURCES.map(String::from));
    }
    let sorted = |d: Directives| -> Directives {
        d.into_iter()
            .map(|(name, mut srcs)| {
                srcs.sort();
                (name, srcs)
            })
            .collect()
    };
    if sorted(parse(dev)?) != sorted(expected) {
        return Err("devCsp differs from csp beyond adding the dev server's origins \
                    to script-src and connect-src"
            .into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Normal cases: the configuration as shipped
// ---------------------------------------------------------------------------

// verifies: LLR-7ymxtn, LLR-hdvy6x, LLR-c88jhh, LLR-df7prq, LLR-p3xwx4, LLR-ausr5q, LLR-uus6ar
#[test]
fn shipped_csp_meets_every_clause() {
    assert_eq!(check(&security()["csp"], &[]), Ok(()));
}

// verifies: LLR-tc5aax
#[test]
fn dev_csp_adds_only_the_dev_server_origins() {
    let sec = security();
    assert_eq!(check(&sec["devCsp"], &DEV_SERVER_SOURCES), Ok(()));
    let dev = sec["devCsp"].as_str().expect("devCsp is a string");
    for origin in DEV_SERVER_SOURCES {
        assert!(dev.contains(origin), "devCsp lacks {origin}");
    }
}

// verifies: LLR-tc5aax
#[test]
fn dev_csp_differs_from_csp_only_by_the_dev_server_origins() {
    let sec = security();
    let shipped = sec["csp"].as_str().expect("csp is a string");
    let dev = sec["devCsp"].as_str().expect("devCsp is a string");
    assert_eq!(check_dev_difference(shipped, dev), Ok(()));
}

// verifies: LLR-7ymxtn, LLR-hdvy6x, LLR-c88jhh, LLR-df7prq, LLR-p3xwx4, LLR-ausr5q, LLR-uus6ar
#[test]
fn fixture_policy_passes_the_checker() {
    assert_eq!(check_as_shipped(FIXTURE), Ok(()));
}

// ---------------------------------------------------------------------------
// Abnormal cases: each weakening is rejected
// ---------------------------------------------------------------------------

// verifies: LLR-7ymxtn
#[test]
fn null_policy_is_rejected() {
    assert!(check(&serde_json::Value::Null, &[]).is_err());
}

// verifies: LLR-7ymxtn
#[test]
fn empty_policy_is_rejected() {
    assert!(check_as_shipped("").is_err());
}

// verifies: LLR-7ymxtn
#[test]
fn default_src_other_than_self_is_rejected() {
    assert!(check_as_shipped(&FIXTURE.replace("default-src 'self'", "default-src *")).is_err());
    assert!(check_as_shipped(&FIXTURE.replace("default-src 'self'; ", "")).is_err());
}

// verifies: LLR-hdvy6x
#[test]
fn unsafe_inline_in_script_src_is_rejected() {
    let p = FIXTURE.replace("script-src 'self'", "script-src 'self' 'unsafe-inline'");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-hdvy6x
#[test]
fn unsafe_eval_in_script_src_is_rejected() {
    let p = FIXTURE.replace("script-src 'self'", "script-src 'self' 'unsafe-eval'");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-ausr5q
#[test]
fn directive_names_are_matched_case_insensitively() {
    // A browser applies the first, upper-case directive; a checker matching
    // names case-sensitively would read only the second and pass the policy.
    let p = format!("SCRIPT-SRC 'unsafe-inline'; {FIXTURE}");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-hdvy6x
#[test]
fn remote_origin_in_script_src_is_rejected() {
    let p = FIXTURE.replace("script-src 'self'", "script-src 'self' https://cdn.example.org");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-p3xwx4
#[test]
fn wildcard_source_is_rejected_in_any_directive() {
    let p = FIXTURE.replace("img-src 'self' data:", "img-src 'self' data: *");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-p3xwx4
#[test]
fn https_scheme_source_is_rejected_in_any_directive() {
    let p = FIXTURE.replace("img-src 'self' data:", "img-src 'self' data: https:");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-c88jhh
#[test]
fn missing_object_src_is_rejected() {
    assert!(check_as_shipped(&FIXTURE.replace("object-src 'none'; ", "")).is_err());
}

// verifies: LLR-c88jhh
#[test]
fn each_none_directive_is_required_to_be_none() {
    for name in NONE_DIRECTIVES {
        let p = FIXTURE.replace(&format!("{name} 'none'"), &format!("{name} 'self'"));
        assert!(check_as_shipped(&p).is_err(), "{name} 'self' was accepted");
    }
}

// verifies: LLR-df7prq
#[test]
fn connect_src_beyond_ipc_is_rejected() {
    let p = FIXTURE.replace("connect-src ipc:", "connect-src ipc: 'self'");
    assert!(check_as_shipped(&p).is_err());
    assert!(check_as_shipped(&FIXTURE.replace("connect-src ipc: http://ipc.localhost; ", "")).is_err());
}

// verifies: LLR-df7prq
#[test]
fn localhost_dev_server_is_rejected_in_shipped_policy() {
    let p = FIXTURE.replace("connect-src ipc:", "connect-src ipc: http://localhost:5173");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-tc5aax
#[test]
fn dev_policy_rejects_an_origin_beyond_the_dev_server() {
    // The development policy may add the dev server's two origins and
    // nothing else: a neighbouring port or a remote origin is refused.
    let near_miss = FIXTURE.replace("connect-src ipc:", "connect-src ipc: http://localhost:5174");
    let remote = FIXTURE.replace("script-src 'self'", "script-src 'self' https://cdn.example.org");
    for policy in [near_miss, remote] {
        assert!(check_as_dev(&policy).is_err(), "{policy}");
    }
}

// verifies: LLR-ausr5q
#[test]
fn repeated_directive_is_rejected() {
    let p = format!("{FIXTURE}; script-src 'self' 'unsafe-inline'");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-uus6ar
#[test]
fn script_src_elem_is_rejected() {
    // CSP Level 3: `script-src-elem` overrides `script-src` for <script>
    // elements, so it would reopen inline script past the script-src check.
    let p = format!("{FIXTURE}; script-src-elem 'self' 'unsafe-inline'");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-uus6ar
#[test]
fn script_src_attr_is_rejected() {
    // `script-src-attr` overrides `script-src` for inline event handlers.
    let p = format!("{FIXTURE}; script-src-attr 'unsafe-inline'");
    assert!(check_as_shipped(&p).is_err());
}

// verifies: LLR-uus6ar
#[test]
fn worker_src_is_rejected() {
    // `worker-src` overrides `script-src` for worker scripts.
    let p = format!("{FIXTURE}; worker-src 'self' blob:");
    assert!(check_as_shipped(&p).is_err());
}

/// The fixture with the dev server's origins added where the dev policy adds
/// them: the one development policy `check_dev_difference` accepts.
fn dev_fixture() -> String {
    FIXTURE
        .replace(
            "script-src 'self'",
            "script-src 'self' http://localhost:5173 ws://localhost:5173",
        )
        .replace(
            "connect-src ipc: http://ipc.localhost",
            "connect-src ipc: http://ipc.localhost http://localhost:5173 ws://localhost:5173",
        )
}

// verifies: LLR-tc5aax
#[test]
fn dev_fixture_is_a_pure_addition_to_the_fixture() {
    assert_eq!(check_dev_difference(FIXTURE, &dev_fixture()), Ok(()));
}

// verifies: LLR-tc5aax
#[test]
fn dev_policy_that_differs_beyond_the_dev_origins_is_rejected() {
    // The reviewer's round-1 probes: each is a development policy the old
    // per-policy check accepted.
    let dev = dev_fixture();
    let cases = [
        ("drops style-src", dev.replace("style-src 'self' 'unsafe-inline'; ", "")),
        ("adds a directive", format!("{dev}; worker-src blob:")),
        ("adds a source", dev.replace("img-src 'self' data:", "img-src 'self' data: blob:")),
        ("omits an origin", dev.replace(" ws://localhost:5173;", ";")),
        (
            "adds the origins elsewhere",
            dev.replace("img-src 'self' data:", "img-src 'self' data: http://localhost:5173"),
        ),
    ];
    for (what, policy) in cases {
        assert!(check_dev_difference(FIXTURE, &policy).is_err(), "{what}: {policy}");
    }
}
