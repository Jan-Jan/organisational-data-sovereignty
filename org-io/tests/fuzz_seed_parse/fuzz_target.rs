#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! verifies: LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy
//! Neither parse panics on any input, and every refusal renders as one of
//! the fixed texts its (variable, rule) pair gives — so no rendering can
//! carry any part of the input. (A substring check would raise false alarms:
//! an input such as "xadecx" shares "adec" with "hexadecimal".)

use org_io::custody::{parse_co_signer, parse_seed, ConfigError, ConfigRule, ConfigVariable};

fn fixed_renderings() -> Vec<(String, String)> {
    let mut renderings = vec![];
    for variable in [ConfigVariable::AdminSeed, ConfigVariable::CoSigner] {
        for rule in [ConfigRule::Empty, ConfigRule::NotHex, ConfigRule::WrongLength, ConfigRule::NotAKey] {
            let error = ConfigError { variable, rule };
            renderings.push((error.to_string(), format!("{error:?}")));
        }
    }
    renderings
}

fn main() {
    let fixed = fixed_renderings();
    bolero::check!().with_type::<String>().for_each(|input: &String| {
        for refusal in [parse_seed(input).err(), parse_co_signer(input).err()].into_iter().flatten() {
            let rendered = (refusal.to_string(), format!("{refusal:?}"));
            assert!(fixed.contains(&rendered), "a refusal rendered outside the fixed set: {rendered:?}");
        }
    });
}
