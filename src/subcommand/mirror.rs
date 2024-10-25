use std::cmp::Ordering;
use std::fmt::Display;

use inquire::formatter::MultiOptionFormatter;
use inquire::ui::Color;
use inquire::ui::RenderConfig;
use inquire::ui::StyleSheet;
use inquire::ui::Styled;
use inquire::MultiSelect;
use oma_console::WRITER;
use oma_mirror::Mirror;
use oma_mirror::MirrorManager;
use oma_pm::apt::AptConfig;

use crate::error::OutputError;
use crate::fl;
use crate::HTTP_CLIENT;

use super::utils::RefreshRequest;

struct MirrorDisplay((Box<str>, Mirror));

impl Display for MirrorDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.0 .1.desc, self.0 .0)?;

        Ok(())
    }
}

pub fn tui(
    no_progress: bool,
    refresh_topic: bool,
    network_threads: usize,
    no_refresh: bool,
) -> Result<i32, OutputError> {
    let mut mm = MirrorManager::new()?;
    let mut mirrors = mm.mirrors_iter()?.collect::<Vec<_>>();
    let enabled = mm.enabled_mirrors();

    sort_mirrors(&mut mirrors, enabled);

    let mirrors = mirrors
        .iter()
        .map(|x| MirrorDisplay((x.0.into(), x.1.to_owned())))
        .collect::<Vec<_>>();

    let formatter: MultiOptionFormatter<MirrorDisplay> =
        &|a| format!("Activating {} mirrors", a.len());
    let render_config = RenderConfig {
        selected_checkbox: Styled::new("✔").with_fg(Color::LightGreen),
        help_message: StyleSheet::empty().with_fg(Color::LightBlue),
        unselected_checkbox: Styled::new(" "),
        highlighted_option_prefix: Styled::new(""),
        selected_option: Some(StyleSheet::new().with_fg(Color::DarkCyan)),
        scroll_down_prefix: Styled::new("▼"),
        scroll_up_prefix: Styled::new("▲"),
        ..Default::default()
    };

    // 空行（最多两行）+ tips (最多两行) + prompt（最多两行）
    let page_size = match WRITER.get_height() {
        0 => panic!("Terminal height must be greater than 0"),
        1..=6 => 1,
        x @ 7..=25 => x - 6,
        26.. => 20,
    };

    let default = (0..enabled.len()).collect::<Vec<_>>();

    let ans = MultiSelect::new(&fl!("select-topics-dialog"), mirrors)
        .with_help_message(&fl!("tips"))
        .with_formatter(formatter)
        .with_default(&default)
        .with_page_size(page_size as usize)
        .with_render_config(render_config)
        .prompt()
        .map_err(|_| anyhow::anyhow!(""))?;

    let set = ans.iter().map(|x| x.0 .0.as_ref()).collect::<Vec<_>>();

    mm.set(&set)?;
    mm.write_status(None)?;

    if !no_refresh {
        RefreshRequest {
            client: &HTTP_CLIENT,
            dry_run: false,
            no_progress,
            limit: network_threads,
            sysroot: "/",
            _refresh_topics: refresh_topic,
            config: &AptConfig::new(),
        }
        .run()?;
    }

    Ok(0)
}

fn sort_mirrors(
    mirrors: &mut Vec<(&str, &Mirror)>,
    enabled: &indexmap::IndexMap<Box<str>, Box<str>>,
) {
    mirrors.sort_unstable_by(|a, b| {
        if enabled.contains_key(a.0) && !enabled.contains_key(b.0) {
            return Ordering::Less;
        } else if !enabled.contains_key(a.0) && enabled.contains_key(b.0) {
            return Ordering::Greater;
        } else {
            return a.0.cmp(b.0);
        }
    });
}

#[test]
fn test_sort() {
    use indexmap::indexmap;
    use indexmap::IndexMap;

    let enabled: IndexMap<Box<str>, Box<str>> = indexmap! {};
    let m1 = Mirror {
        desc: "baka".into(),
        url: "bala".into(),
    };

    let m2 = Mirror {
        desc: "baka".into(),
        url: "bala".into(),
    };

    let m3 = Mirror {
        desc: "baka".into(),
        url: "bala".into(),
    };

    let mut mirrors = vec![("b", &m1), ("a", &m2)];

    sort_mirrors(&mut mirrors, &enabled);

    assert_eq!(
        mirrors.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec!["a", "b"]
    );

    let enabled: IndexMap<Box<str>, Box<str>> = indexmap! {"c".into() => "baka".into()};
    let mut mirrors = vec![("b", &m1), ("a", &m2), ("c", &m3)];

    sort_mirrors(&mut mirrors, &enabled);

    assert_eq!(
        mirrors.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec!["c", "a", "b"]
    );
}
