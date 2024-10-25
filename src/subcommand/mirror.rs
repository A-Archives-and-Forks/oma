use std::cmp::Ordering;

use indexmap::indexmap;
use indexmap::IndexMap;
use oma_mirror::Mirror;
use oma_mirror::MirrorManager;

use crate::error::OutputError;

struct MirrorsDisplay<'a>(Vec<(&'a str, &'a Mirror)>);

pub fn tui() -> Result<i32, OutputError> {
    let mm = MirrorManager::new()?;
    let mut mirrors = mm.mirrors_iter()?.collect::<Vec<_>>();
    let enabled = mm.enabled_mirrors();

    sort_mirrors(&mut mirrors, enabled);

    todo!()
}

fn sort_mirrors(
    mirrors: &mut Vec<(&str, &Mirror)>,
    enabled: &indexmap::IndexMap<Box<str>, Box<str>>,
) {
    mirrors.sort_unstable_by(|a, b| {
        if enabled.contains_key(a.0) && !enabled.contains_key(b.0) {
            return Ordering::Greater;
        } else if !enabled.contains_key(a.0) && enabled.contains_key(b.0) {
            return Ordering::Less;
        } else {
            return b.0.cmp(a.0);
        }
    });
}

#[test]
fn test_sort() {
    let enabled: IndexMap<Box<str>, Box<str>> = indexmap! {};
    let m1 = Mirror {
        desc: "baka".into(),
        url: "bala".into(),
    };

    let m2 = Mirror {
        desc: "baka".into(),
        url: "bala".into(),
    };

    let mirrors = vec![("b", &m1), ("a", &m2)];

    sort_mirrors(mirrors, &enabled);

    dbg!(mirrors);
}
